use chrono::{Duration, NaiveDateTime, NaiveTime};

use string_builder::Builder;

use serde::{Serialize, Deserialize};

use super::resources::{Resource, ResourceType};
use super::util::{get_space_indent, get_duration_in_hours, convert_string_to_duration};

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
pub struct ProductionPhaseTemplate {
    pub description: String,
    pub id: String,
    pub order: usize,

    #[serde(rename="resourcesNeeded")]
    #[serde(default="Vec::new")]
    pub resources_needed: Vec<ResourceType>,

    #[serde(rename="color")]
    #[serde(default = "String::new")]
    color_hex: String,

    #[serde(rename="defaultDuration")]
    #[serde(default = "String::new")]
    default_duration: String
}

impl ProductionPhaseTemplate {
    pub fn default_duration(&self) -> Option<Duration> {
        convert_string_to_duration(&self.default_duration[..])
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PhaseInstanceSpec {
    #[serde(default = "String::new")]
    pub description: String,

    pub template: String,

    #[serde(rename = "duration")]
    #[serde(default = "String::new")]
    pub duration_string: String
}

impl PhaseInstanceSpec {
    pub fn duration(&self) -> Option<Duration> {
        match self.duration_string.is_empty() {
            true => None,
            false => convert_string_to_duration(&self.duration_string[..])
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct PhaseInstance {
    pub id: usize,
    pub description: String,
    pub color_hex: String,
    pub duration: Duration,
    pub dependencies: Vec<usize>,
    pub start_date: NaiveDateTime,
    pub resources_used: Vec<Resource>
}

impl PhaseInstance {
    /// Construct a new `PhaseInstance` object.
    ///
    /// # Arguments
    /// - `id`: The id of the new `PhaseInstance` object. Should be unique within the entire
    ///   [ProductionPhase](super::ProductionSchedule), however, this is not automatically
    ///   enforced.
    /// - `description`: A `String` that describes the phase.
    /// - `color_hex`: A `String` containing a color representing the phase, in hexadecimal format.
    /// - `duration`: A [Duration](chrono::Duration) specifying the length of the phase.
    /// - `start_date`: A [NaiveDateTime](chrono::NaiveDateTime) specifying when the
    ///   PhaseInstance` will begin.
    /// - `resources`: A `Vec` of `Resource` objects that will be used as part of this
    ///   `PhaseInstance`. These resources should be marked as not available during the duration of
    ///   this `PhaseInstance` within the
    ///   [ResourceTracker](super::resources::ResourceTracker).
    ///
    /// # Returns
    /// - A new `PhaseInstance` object.
    ///
    pub fn new(id: usize, description: String, color_hex: String, duration: Duration,
               start_date: NaiveDateTime, resources: Vec<Resource>) -> Self {
        PhaseInstance{
            description: description,
            id: id,
            color_hex: color_hex,
            duration: duration,
            dependencies: vec![],
            start_date: start_date,
            resources_used: resources
        }
    }

    pub fn add_dependency(&mut self, dep: usize) {
        if !self.dependencies.clone().into_iter().any(|d| d == dep) {
            let mut dependencies: Vec<usize> = self.dependencies.clone();
            dependencies.push(dep);
            dependencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
            self.dependencies = dependencies;
        }
    }

    /// Retrieve a `String` representing this `PhaseInstance` in PLA format.
    ///
    /// # Arguments
    /// - `self`: A borrowed reference to this `PhaseInstance`.
    /// - `initial_indent`: A `usize` indicating the indentation to use for the output `String`.
    ///
    /// # Returns
    /// - A `String` containing the data from this `PhaseInstance` in PLA format.
    ///
    pub fn get_string_in_pla_format(&self, initial_indent: usize) -> String {
        // If the time is set to start at midnight, then let's just output the date.
        let mut start_date_as_string: String = self.start_date.format("%Y-%m-%d %H").to_string();
        let midnight: NaiveTime = NaiveTime::from_hms(0, 0, 0);
        if self.start_date.time() == midnight {
            start_date_as_string = self.start_date.date().to_string();
        }

        let mut builder = Builder::default();
        builder.append(format!("{}[{}] {}\n", get_space_indent(initial_indent), self.id, self.description));
        builder.append(format!("{}start {}\n", get_space_indent(initial_indent + 1), start_date_as_string));
        builder.append(format!("{}color {}\n", get_space_indent(initial_indent + 1), self.color_hex));
        builder.append(format!("{}duration {}\n", get_space_indent(initial_indent + 1), get_duration_in_hours(self.duration)));

        for next_resource in &self.resources_used {
            builder.append(format!("{}res {}\n", get_space_indent(initial_indent + 1), next_resource.name));
        }

        for next_dependency in self.dependencies.iter() {
            builder.append(format!("{}dep {}\n", get_space_indent(2), next_dependency));
        }

        builder.append("\n");

        builder.string().unwrap()
    }
}
