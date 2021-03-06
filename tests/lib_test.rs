use std::fs;

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};

use chronogrog::ProductionSchedule;
use chronogrog::resources::ResourceType;

use chronogrog::util::get_json_data_from_file;

#[test]
fn it_should_load_a_json_file_into_a_new_production_schedule() {

    let ps = ProductionSchedule::new(&get_json_data_from_file("tests/fixtures/simple_prod_schedule.json").unwrap()[..]);

    assert_eq!("Simple Production Schedule", ps.name);
    assert_eq!(1, ps.id);
    assert_eq!("calendar", ps.timeline.configuration);

    let current_date = NaiveDateTime::parse_from_str("2020-01-01 00:00:00", "%Y-%m-%d %H:%M:%S");
    assert_eq!(current_date, ps.timeline.start_date());

    // Verify that the recipe spec with name "Damned Squirrel Mk. II" appears and has a start date
    // of 01-01-20.
    println!("{:?}", ps.recipe_specs);
    let damned_squirrel = ps.recipe_specs.clone().into_iter()
                                         .find(|rs| rs.name == "Damned Squirrel Mk. II").unwrap();

    assert_eq!(NaiveDateTime::new(NaiveDate::from_ymd(2020, 1, 1), NaiveTime::from_hms(0, 0, 0)),
               damned_squirrel.start_date().unwrap());

}

#[test]
fn the_simple_production_schedule_file_should_have_three_phases() {
    let ps = ProductionSchedule::new(&get_json_data_from_file("tests/fixtures/simple_prod_schedule.json").unwrap()[..]);

    assert_eq!("Planning", ps.phase_templates[0].description);

    assert_eq!(Some(Duration::hours(1)), ps.phase_templates[0].default_duration());

    assert_eq!("Primary Fermentation", ps.phase_templates[2].description);
    assert_eq!(Some(Duration::days(10)), ps.phase_templates[2].default_duration());

    assert_eq!("Secondary Fermentation", ps.phase_templates[3].description);
    assert_eq!(Some(Duration::weeks(4)), ps.phase_templates[3].default_duration());

    assert_eq!(Some(ps.phase_templates[3].clone()), ps.get_phase_by_id("secondary"));
}

#[test]
fn the_simple_production_schedule_should_include_six_resources() {
    let ps = ProductionSchedule::new(&get_json_data_from_file("tests/fixtures/simple_prod_schedule.json").unwrap()[..]);

    assert_eq!(9, ps.resources().len());

    // There should be a kettle in the resources
    let resources = &ps.resources();
    let mut found = false;
    for next in resources {
        match next.resource_type {
            ResourceType::Kettle => {
                found = true;
            }
            _ => { found = found; }
        };
    }

    assert!(found);

    // Resource with id 1 should exist and be of type 'fermentor'
    match &ps.get_resource_by_id(1) {
        Some(x) => {
            assert_eq!(ResourceType::Fermentor, x.resource_type);
        },
        None => { assert!(false) }
    }
}

#[test]
fn it_should_be_able_to_retrieve_recipes_by_name_and_id() {
    let ps = ProductionSchedule::new(&get_json_data_from_file("tests/fixtures/simple_prod_schedule.json").unwrap()[..]);

    let damned_squirrel = ps.get_recipe_by_name("Damned Squirrel Mk. II").unwrap();
    assert_eq!(damned_squirrel.name, "Damned Squirrel Mk. II");
}

#[test]
fn it_should_be_able_to_retrieve_an_available_resource_by_type() {
    let ps = ProductionSchedule::new(&get_json_data_from_file("tests/fixtures/simple_prod_schedule.json").unwrap()[..]);

    let res = ps.get_available_resource_by_type(ResourceType::Kettle).unwrap();

    assert_eq!("Large Kettle", res.name);

    let res2 = ps.get_available_resource_by_type(ResourceType::Other("nitrogastank".to_string()));

    assert_eq!(None, res2);
}

#[test]
fn it_should_be_able_to_convert_a_simple_bpd_file_to_a_pla_file() {
    let ps = ProductionSchedule::new(&get_json_data_from_file("tests/fixtures/simple_prod_schedule.json").unwrap()[..]);

    let pla_format: String = ps.get_string_in_pla_format();

    let contents = fs::read_to_string("tests/fixtures/simple_prod_schedule.pla")
                         .expect("Something went wrong reading the file");
    assert_eq!(contents, pla_format);
}

#[test]
fn it_should_be_able_to_convert_a_complicated_bpd_file_to_a_pla_file() {
    let ps = ProductionSchedule::new(&get_json_data_from_file("tests/fixtures/complicated_prod_schedule.json").unwrap()[..]);

    let pla_format: String = ps.get_string_in_pla_format();

    let contents = fs::read_to_string("tests/fixtures/complicated_prod_schedule.pla")
                         .expect("Something went wrong reading the file");
    assert_eq!(contents, pla_format);
}

#[test]
#[should_panic]
fn it_should_panic_on_an_unparseable_json_file() {
    ProductionSchedule::new(&get_json_data_from_file("tests/fixtures/bad_production_schedule.json").unwrap()[..]);
}
