use std::default::Default;
use std::io::prelude::*;
use std::iter::Iterator;

extern crate chrono;
use chrono::{Duration, NaiveDateTime};
use chrono::format::ParseError;

extern crate chrono_period;
use chrono_period::NaivePeriod;

use serde::{Deserialize, Serialize};

extern crate string_builder;
use string_builder::Builder;

pub mod util;
use util::get_naive_date_time_from_string;

pub mod resources;
use resources::Resource;
use resources::ResourceTracker;
use resources::ResourceType;

pub mod phases;
use phases::PhaseInstance;
use phases::ProductionPhaseTemplate;

pub mod recipes;
use recipes::RecipeSpec;
use recipes::Recipe;

#[derive(Serialize, Deserialize)]
/// Configuration options for the timeline of the production schedule.
///
/// At some point in the future, we're going to allow for the gantt-chart-creation software to take
/// configuration options that allow it to either start at a specific date and start counting from
/// that date , or start at a specific number, and just count generic days. Currently, since the
/// gantt-chart-creation part of the application does not include this, it's not implemented other
/// than to create these configuration variables as part of deserialization.
///
pub struct ProductionTimeline {
    pub configuration: String,
    start: String
}

impl ProductionTimeline {
    /// Retrieve the start date, as a `NaiveDateTime`.
    ///
    /// # Returns
    /// * A `Result` containing either the starting date for the production schedule, as a
    ///   [NaiveDateTime](chrono::NaiveDateTime), if the parsing was successful, or a `ParseError`
    ///   explaining what happened.
    ///
    pub fn start_date(&self) -> std::result::Result<NaiveDateTime, ParseError> {
        get_naive_date_time_from_string(&self.start[..])
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProductionSchedule {
    pub name: String,
    pub id: usize,
    pub timeline: ProductionTimeline,

    #[serde(rename="phaseTemplates")]
    pub phase_templates: Vec<ProductionPhaseTemplate>,

    // XXX_jwir3: Note that this is _only_ for deserialization. It is not for usage after the
    //            object has been deserialized from JSON, because resources are tracked within the
    //            ResourceTracker instance.
    resources: Vec<Resource>,

    #[serde(skip_serializing, skip_deserializing)]
    recipes: Vec<Recipe>,

    #[serde(rename="recipes")]
    pub recipe_specs: Vec<RecipeSpec>,

    #[serde(skip_serializing, skip_deserializing)]
    last_id_used: usize,

    #[serde(skip_serializing, skip_deserializing, default = "ResourceTracker::new")]
    tracker: ResourceTracker
}

impl ProductionSchedule {
    pub fn new(json_data: &str) -> Self {
        let result: serde_json::Result<ProductionSchedule> = serde_json::from_str(json_data);
        match result {
            Ok(mut x) => {
                x.init();
                x
            },
            Err(e) => {
                panic!("Unable to parse due to: {}", e);
            }
        }
    }

    pub fn init(&mut self) {
        self.last_id_used = 0;
        self.track_resources();
        self.verify_recipe_start_dates();
        self.rebuild_recipes_from_specs();
    }

    pub fn resources(&self) -> Vec<Resource> {
        self.tracker.get_all_tracked_resources()
    }

    pub fn get_phase_by_id(&self, id: &str) -> Option<ProductionPhaseTemplate> {
        for next_phase in &self.phase_templates {
            if next_phase.id == id {
                return Some(next_phase.clone());
            }
        }

        None
    }

    pub fn get_resource_by_id(&self, id: usize) -> Option<Resource> {
        for next_res in &self.resources {
            if next_res.id == id {
                return Some(next_res.clone());
            }
        }

        None
    }

    pub fn get_available_resource_by_type(&self, resource_type: ResourceType) -> Option<Resource> {
        let resources: Vec<Resource> = self.resources.clone();
        if resources.iter().any(|x| x.resource_type == resource_type) {
            return resources.into_iter()
                            .filter(|x| x.resource_type == resource_type)
                            .next();
        }

        None
    }

    // pub fn get_recipe_by_id(&mut self, id: usize) -> Option<Recipe> {
    //     let mut spec: Option<RecipeSpec> = None;
    //     let recipes = self.recipe_specs.clone();
    //     for next_recipe in recipes {
    //         if next_recipe.id == id {
    //             spec = Some(next_recipe);
    //             break;
    //         }
    //     }
    //
    //     match spec {
    //         Some(x) => Some(Recipe {
    //                      id: self.get_next_id(),
    //                      name: x.name.clone(),
    //                      color: x.color(),
    //                      phases: vec![]
    //                  }),
    //         None => None
    //     }
    // }

    pub fn get_recipe_by_name(&self, name: &str) -> Option<Recipe> {
        for next_recipe in &self.recipes {
            if next_recipe.name == name {
                return Some(next_recipe.clone());
            }
        }

        None
    }

    pub fn get_recipe_iterator(&self) -> std::slice::Iter<Recipe> {
        self.recipes.iter()
    }

    pub fn get_string_in_pla_format(&self) -> String {
        let mut builder = Builder::default();
        for next_recipe in self.get_recipe_iterator() {
            builder.append(next_recipe.get_string_in_pla_format(1));

            for next_phase in next_recipe.get_phase_iterator() {
                builder.append(next_phase.get_string_in_pla_format(1));
            }
        }

        let final_pla: String = builder.string().unwrap();

        // Remove the last newline at the end of the file, as it's unnecessary
        final_pla[..final_pla.len() - 1].to_string()
    }

    pub fn write_pla_file(&self, mut output_stream: Box<dyn Write>) -> std::io::Result<()> {
        let pla_data = self.get_string_in_pla_format();

        output_stream.write_all(pla_data.as_bytes())
    }

    fn get_next_id(&mut self) -> usize {
        self.last_id_used += 1;

        self.last_id_used
    }

    fn verify_recipe_start_dates(&mut self) {
        let mut new_recipe_vec: Vec<RecipeSpec> = vec![];

        self.recipe_specs.clone().into_iter().for_each(|recipe_spec| {
            match recipe_spec.start_string.clone() {
                Some(_x) => new_recipe_vec.push(recipe_spec),
                None => {
                    let new_recipe_spec = RecipeSpec {
                        name: recipe_spec.name,
                        color_hex: recipe_spec.color_hex,
                        phase_specs: recipe_spec.phase_specs,
                        start_string: Some(self.timeline.start.clone())
                    };

                    new_recipe_vec.push(new_recipe_spec);
                }
            }

            self.recipe_specs = new_recipe_vec.clone();
        })
    }

    fn rebuild_recipes_from_specs(&mut self) {
        let mut recipes_vec = vec![];
        let recipes = self.recipe_specs.clone();
        for next_recipe_spec in recipes {
            let recipe_start_date: NaiveDateTime = match next_recipe_spec.start_date() {
                Ok(x) => x,

                // If the next line causes an error, there's no telling what we can do...
                Err(_e) => self.timeline.start_date().unwrap()
            };

            let mut recipe_template: Recipe = Recipe {
                id: self.get_next_id(),
                name: next_recipe_spec.name.clone(),
                color: next_recipe_spec.color_hex.clone(),
                phases: vec![],
                start_date: recipe_start_date
            };

            recipe_template.phases = self.rebuild_phases_from_specs(&next_recipe_spec);

            recipes_vec.push(recipe_template);
        }

        self.recipes = recipes_vec;
    }

    /// Rebuild all `PhaseInstance's from a set of [RecipeSpec](chronogrog::recipes::RecipeSpec)
    /// and  a [ProductionPhaseTemplate](chrono::phases::ProductionPhaseTemplate).
    ///
    /// This places all the necessary data into the data structures necessary to convert the
    /// JSON specification to a PLA formatted file.
    ///
    /// # Arguments
    /// - `self` : A _mutable_ reference to this
    ///   [ProductionSchedule](chronogrog::ProductionSchedule). It needs to be mutable because
    ///   internal references are updated as part of this function call.
    /// - `recipe_spec` : A borrowed reference to a [RecipeSpec](chronogrog::recipes::RecipeSpec)
    ///    defining the containing recipe specification that will be used to construct the
    ///    instances of production phases.
    ///
    /// # Returns
    /// - A `Vec` of `PhaseInstance` objects.
    fn rebuild_phases_from_specs(&mut self, recipe_spec: &RecipeSpec) -> Vec<PhaseInstance> {
        let mut phases: Vec<PhaseInstance> = vec![];

        // The start date of the next phase
        let mut next_start_date: NaiveDateTime = recipe_spec.start_date().unwrap();

        for next_spec in recipe_spec.phase_specs.iter() {
            let id: usize = self.get_next_id();

            // If the duration is specified in the spec, use that duration.
            // Otherwise, use the default duration by looking up from the template.
            let mut dur: Option<Duration> = next_spec.duration();
            dur = match dur {
                Some(x) => Some(x),
                None => {
                    let template: ProductionPhaseTemplate = self.get_phase_by_id(&next_spec.template[..]).unwrap();
                    template.default_duration()
                }
            };

            // The production phase template we're going to use to construct this instance.
            let template: ProductionPhaseTemplate = self.get_phase_by_id(&next_spec.template[..]).unwrap();

            // If the description is specified in the spec, use that description.
            // Otherwise, use the description by looking up from the template.
            let mut description: String = next_spec.description.clone();

            description = match description.is_empty()  {
                true => {
                    template.description.clone()
                },
                false => description
            };

            let duration: Duration = match dur {
                Some(x) => x,

                // Default to a single day if nothing else works
                None => Duration::days(1)
            };

            let mut resources_used : Vec<Resource> = vec![];
            let mut result_start_date;

            // We need to look through all the resources to determine if we have to push the start
            // date back due to resource allocation constraints.
            for next_resource_type in template.resources_needed.clone() {
                let requested_start_date = next_start_date.clone();

                let requested_period = NaivePeriod::from_start_duration(requested_start_date,
                                                                        duration);

                result_start_date = match self.tracker
                  .get_next_available_resource_date_for_type_over_period(&next_resource_type,
                                                                         requested_period) {
                  Some(date) => date,
                  None => panic!("{} {:?}, {} {:?}", "No resources of type", next_resource_type,
                                 "which is required by phase", template.description.clone())
                };

                // If we can't allocate a resource in the given timeframe, we need to push back
                // the start date of the phase.
                if result_start_date.timestamp() > next_start_date.timestamp() {
                    next_start_date = result_start_date;
                }
            }

            for next_resource_type in template.resources_needed {
                // Allocate the resource
                let allocation_period = NaivePeriod::from_start_duration(next_start_date,
                                                                         duration);

                let allocated_resource
                  = self.tracker.allocate_resource_of_type_for_period(&next_resource_type,
                                                                      allocation_period);

                if (allocated_resource).is_none() {
                    panic!("Unable to allocate a resource of type {:?}", next_resource_type);
                }

                // Put the allocated resource into the vector
                resources_used.push(allocated_resource.unwrap().clone());
            }

            phases.push(PhaseInstance::new(id, description, recipe_spec.color_hex.clone(),
                                           duration, next_start_date, resources_used));
            next_start_date = next_start_date + duration;
        }

        // We have to actally run through the phases from the back and add the next phase id
        // as a dependency to the previous phase id. This is a weird nuance of pla that tasks X
        // that are dependent on some task Y are actually defined in the definition of Y, not X. It
        // basically means you have to specify that there will be defined a task with id X, but
        // that task hasn't been defined yet.
        let mut phases_new : Vec<PhaseInstance> = vec![];
        let mut prev_phase_id : usize = 0;
        for next_phase in phases.iter().rev() {
            let mut phase = next_phase.clone();
            if prev_phase_id > 0 {
                phase.add_dependency(prev_phase_id);
            }

            prev_phase_id = phase.id;

            phases_new.push(phase);
        }

        phases_new.into_iter().rev().collect()
    }

    fn track_resources(&mut self) {
        self.resources.clone().into_iter().for_each(|e| self.tracker.track_resource(e));
    }
}
