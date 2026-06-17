use std::collections::HashMap;

use rust_qsim::simulation::id::Id;
use rust_qsim::simulation::pt::TransitStopFacility;
use rust_qsim::simulation::scenario::Coordinate;
use rust_qsim::simulation::time::SimTime;
use nohash_hasher::IntMap;

const WALK_SPEED: f64 = 1.11;

/// Euclidean distance between two coordinates
fn euclidean(a: &Coordinate, b: &Coordinate) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

/// Convert distance (meters) into walking time
fn walk_time(distance: f64) -> SimTime {
    SimTime::from_secs((distance / WALK_SPEED) as u64)
}
pub fn print_footpath_graph(
    graph: &HashMap<Id<TransitStopFacility>, Vec<(Id<TransitStopFacility>, SimTime)>>
) {
    println!("--- Footpath Graph ---");

    for (from, edges) in graph {
        if edges.is_empty() {
            continue;
        }

        print!("Stop {:?} -> ", from);

        let mut first = true;

        for (to, time) in edges {
            if !first {
                print!(", ");
            }
            first = false;

            // assumes SimTime has a way to get seconds; adjust if needed
            print!("({:?}, {:?})", to, time);
        }

        println!();
    }

    println!("--- End Footpaths ---");
}
pub fn build_footpath_graph(
    stops: &IntMap<Id<TransitStopFacility>, TransitStopFacility>,
    max_distance: f64,
) -> HashMap<Id<TransitStopFacility>, Vec<(Id<TransitStopFacility>, SimTime)>> {
    let mut graph: HashMap<
        Id<TransitStopFacility>,
        Vec<(Id<TransitStopFacility>, SimTime)>
    > = HashMap::new();

    let stop_list: Vec<_> = stops.values().collect();

    for i in 0..stop_list.len() {
        let from = &stop_list[i];

        for j in 0..stop_list.len() {
            if i == j {
                continue;
            }

            let to = &stop_list[j];

            let dist = euclidean(&from.coord, &to.coord);

            if dist <= max_distance {
                graph
                    .entry(from.id.clone())
                    .or_insert_with(Vec::new)
                    .push((
                        to.id.clone(),
                        walk_time(dist),
                    ));
            }
        }
    }

    graph
}