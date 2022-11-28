use std::cmp::Ordering;

use irisdata::{Species, IRIS_DATA};
use joinable::{JoinableGrouped, RHS};

#[derive(Debug)]
struct IrisData {
    species: Species,
    common_name: &'static str,
    average_sepal_length: f32,
    average_sepal_width: f32,
    average_petal_length: f32,
    average_petal_width: f32,
}

fn main() {
    let common_names = [
        (Species::IrisVersicolor, "blue flag"),
        (Species::IrisVersicolor, "harlequin blueflag"),
        (Species::IrisVersicolor, "larger blue flag"),
        (Species::IrisVersicolor, "northern blue flag"),
        (Species::IrisVersicolor, "poison flag"),
        (Species::IrisVirginica, "Virginia blueflag"),
        (Species::IrisVirginica, "Virginia iris"),
        (Species::IrisVirginica, "great blue flag"),
        (Species::IrisVirginica, "southern blue flag"),
    ];

    let joined = common_names
        .iter()
        .inner_join_grouped(RHS::new_unsorted(&IRIS_DATA[..]), |(lhs_species, _), r| {
            if *lhs_species == r.species {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        })
        .map(|(lhs, grp)| IrisData {
            species: lhs.0,
            common_name: lhs.1,
            average_sepal_length: grp.iter().map(|i| i.sepal_length).sum(),
            average_sepal_width: grp.iter().map(|i| i.sepal_width).sum(),
            average_petal_length: grp.iter().map(|i| i.petal_length).sum(),
            average_petal_width: grp.iter().map(|i| i.petal_width).sum(),
        })
        .collect::<Vec<_>>();

    println!("{joined:#?}");
}
