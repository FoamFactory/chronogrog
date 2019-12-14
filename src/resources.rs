use chrono::Duration;
use chrono::NaiveDate;

use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Clone, Debug, PartialEq)]

/// Type of a particular resource.
pub enum ResourceType {
    /// A unit for storage during fermentation.
    Fermentor,

    /// A resource for heating water and boiling sweet wort.
    Kettle,

    /// A place to convert raw grain into sweet wort.
    MashTun,

    /// A place for separating the liquid and solid components of a mash. In homebrewing, this is
    /// often synonymous with a hot liquor tank, which is a place for holding hot water.
    LauterTun,

    /// A place for carbonating, aging, and serving beer.
    Keg,

    /// A place to put kegs in order to refrigerate.
    Kegerator,

    /// A resource type that has not yet been added to the standard enum. The "real" type of the
    /// resource, for the purposes of serialization and deserialization, will be contained in the
    /// string variable present in the enum instance.
    Other(String)
}

impl From<&str> for ResourceType {
    /// Convert from a string slice (`&str`) to a `ResourceType`.
    ///
    /// # Arguments
    /// * `res`: A string slice (`&str`) that will be converted.
    ///
    /// # Returns
    ///
    /// * A `ResourceType` corresponding to the appropriate `&str` reference.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chronogrog::resources::ResourceType;
    /// let s = "fermentor";
    ///
    /// assert_eq!(ResourceType::Fermentor, ResourceType::from(s));
    /// ```
    fn from(res: &str) -> Self {
        match res {
            "fermentor" => ResourceType::Fermentor,
            "kettle" => ResourceType::Kettle,
            "mashtun" => ResourceType::MashTun,
            "lautertun" => ResourceType::LauterTun,
            "keg" => ResourceType::Keg,
            "kegerator" => ResourceType::Kegerator,
            _ => ResourceType::Other(res.to_string())
        }
    }
}

impl From<String> for ResourceType {
    /// Convert from a `String` to a `ResourceType`.
    ///
    /// # Arguments
    /// * `res`: A `String` to be converted.
    ///
    /// # Returns
    ///
    /// * A `ResourceType` corresponding to the appropriate `&str` reference.
    ///
    /// # Notes
    ///
    /// This is a convenience function that converts the `String` to a string slice (`&str`) using
    /// `&res[..]` and calls the other variant of `From::<&str>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use chronogrog::resources::ResourceType;
    /// let s = "fermentor".to_string();
    ///
    /// assert_eq!(ResourceType::Fermentor, ResourceType::from(s));
    /// ```
    fn from(res: String) -> Self {
        ResourceType::from(&res[..])
    }
}

/// Implementation of `serde_json::de::Deserializer` for `ResourceType` instances.
impl<'de> Deserialize<'de> for ResourceType {
    /// Deserialize from JSON into a `ResourceType`.
    ///
    /// # Arguments
    /// * `deserializer` - A `serde::de::Deserializer` that will be used for deserialization.
    ///
    /// # Returns
    /// * A `Result` containing a `ResourceType` as deserialized from JSON, or an `Error` if the
    ///   deserialization failed.
    ///
    /// # Errors
    /// * An `Error`, if the deserialization failed (most commonly associated with malformed JSON).
    ///   The string component of the `Error` will explain why the deserialization failed, and at
    ///   what location in the JSON the failure occurred.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> {
            let s = String::deserialize(deserializer)?;
            Ok(ResourceType::from(s.as_str()))
    }
}

/// Serialize to JSON from a `ResourceType`.
///
/// # Arguments
/// * `serializer` - A `serde::ser::Serializer` that will be used for serialization.
///
/// # Returns
/// * A `Result` containing a `String` containing serialized JSON, or an `Error` if the
///   serialization failed.
///
/// # Errors
/// * An `Error`, if the serialization failed.

impl Serialize for ResourceType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer
    {
        serializer.serialize_str(match *self {
            ResourceType::Fermentor => "fermentor",
            ResourceType::Kettle => "kettle",
            ResourceType::MashTun => "mashtun",
            ResourceType::LauterTun => "lautertun",
            ResourceType::Keg => "keg",
            ResourceType::Kegerator => "kegerator",
            ResourceType::Other(ref other) => other
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]

/// A piece of equipment that must be used in order to produce a `Recipe`.
pub struct Resource {
    pub id: usize,
    pub name: String,

    #[serde(rename="type")]
    pub resource_type: ResourceType,

    #[serde(rename="capacity")]
    pub capacity_str: String,
}

#[derive(Clone)]
/// A `Resource` that may be allocated (and thus not usable).
///
/// The majority of the time, the `Resource` bound to this object will be in use, and thus is not
/// available for use. However, at the border of expiration, the object may not yet have been freed
/// within the `ResourceTracker` (hence, the name 'PossiblyAllocated').
pub struct PossiblyAllocatedResource {
    /// A borrowed reference to the `Resource` that is currently in use.
    pub resource: Resource,

    /// The `NaiveDate` upon which the `Resource` will be free again. Given the current
    /// `NaiveDate`, `c`, if `free_date` is on or before `c`, the `Resource` should be freed.
    pub free_date: NaiveDate
}

/// Tracking mechanism for `Resource`s.
///
/// All usage of `Resource` objects shoiuld be tracked through this data structure. This provides
/// an API for retrieving an unused `Resource` of a given type.
///
/// This data structure tracks used and unused `Resource` objects. For `Resource`s that are
/// currently in use (and thus cannot be allocated to a new task), each object is tracked using the
/// `AllocatedResource` structure.
///
/// For unused `Resources`, a borrowed reference to the individual `Resource` is available.
pub struct ResourceTracker {
    /// All `Resource` objects that are not currently in use
    free_resources: Vec<Resource>,

    /// Wrapped `Resource` objects that are in use (since the last refresh)
    allocated_resources: Vec<PossiblyAllocatedResource>
}

impl ResourceTracker {
    /// Create a new `ResourceTracker` to track [Resource](Resource)
    /// objects.
    ///
    /// `Resource` objects are tracked as _in use_ or _free_. Free resources can be allocated so
    /// that the date at which they are once again free can be tracked. This is necessary for
    /// scheduling resources. `Resource`s that are in use cannot be scheduled a second time, but
    /// they can be queried for when they will be free.
    ///
    /// # Notes
    ///
    /// The `ResourceTracker` created is initially empty. It is necessary to add `Resource`s
    /// manually using the [track_resource](ResourceTracker::track_resource) method.
    //
    pub fn new() -> Self {
        ResourceTracker {
            free_resources: vec![],
            allocated_resources: vec![]
        }
    }

    /// Track a `Resource` using this `ResourceTracker`.
    ///
    /// Calling this method moves the `Resource` in question to be owned by this `ResourceTracker`.
    /// Thus, the lifetime of the `Resource` is bound to the lifetime of this `ResourceTracker`,
    /// and only borrowed references should be used.
    ///
    /// # Arguments
    ///
    /// * `res`: A  `Resource` object to track within this `ResourceTracker`.
    ///
    pub fn track_resource(&mut self, res: Resource) {
        self.free_resources.push(res);
    }

    /// Retrieve the next [NaiveDate](chrono::NaiveDate) at which a `Resource` of a specific
    /// `ResourceType` will be free.
    ///
    /// # Arguments
    ///
    /// * `resource_type`: The [ResourceType](ResourceType) to query for.
    ///
    /// # Returns
    ///
    /// * An `Option` containing one of the following values:
    ///   * `Some`: Contains an instance of type [NaiveDate](chrono::NaiveDate) that represents the
    ///     closest date at which a `Resource` of type `resource_type` will be free, if there is
    ///     at least one `Resource` of type `resource_type` allocated.
    ///   * `None`: If there are no `Resource`s of type `resource_type` allocated.
    ///
    pub fn next_available_resource_date_for_type(&self, resource_type : ResourceType)
      -> Option<NaiveDate> {
          // XXX_jwir3: This should probably do something if there are resources of type
          //            resource_type already free.
      let allocated_of_type : Vec<PossiblyAllocatedResource> =
        self.allocated_resources.clone().into_iter()
                                .filter(|r| r.resource.resource_type == resource_type)
                                .collect();
      if allocated_of_type.is_empty() {
          return None;
      }

      let next_free = allocated_of_type.into_iter()
                                       .min_by(|x, y| x.free_date.cmp(&y.free_date))
                                       .map(|x| x.free_date);
      next_free
    }

    /// Query for a `Resource` of a specific `ResourceType`, and mark the next free instance of
    /// this type as _in use_ for a given period of time from a starting date.
    ///
    /// # Arguments
    /// * `resource_type`: A [ResourceType](ResourceType) to query for.
    /// * `start_date`: The [NaiveDate](chrono::NaiveDate) at which the allocation of the first
    ///   encountered instance of an unallocated `Resource` of type `resource_type` should begin.
    /// * `duration`: A [Duration](chrono::Duration) for which the `Resource` will be allocated.
    ///
    /// # Returns
    /// - An `Option` containing either a `Resource` (a cloned version of the allocated
    ///   `Resource`), if there is a free `Resource` of type `resource_type`; otherwise `None`.
    ///
    pub fn allocate_resource_of_type_for_duration(&mut self, resource_type: ResourceType,
                                                  start_date: NaiveDate,
                                                  duration: Duration) -> Option<Resource> {
        let removed = self.free_resources.iter()
                                         .position(|n| n.resource_type == resource_type)
                                         .map(|e| self.free_resources.remove(e));

        match removed {
            Some(x) => {
                let id : usize = x.id;
                self.allocated_resources.push(PossiblyAllocatedResource {
                resource: x,
                free_date: start_date.checked_add_signed(duration).unwrap()
            });

            self.allocated_resources.clone()
                                    .into_iter()
                                    .map(|r| r.resource)
                                    .find(|r| r.id == id)
            },
            None => None
        }
    }
}
