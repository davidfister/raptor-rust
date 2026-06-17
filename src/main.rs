mod footpaths;

use std::path::Path;
use rust_qsim::simulation::pt::{TransitDeparture, TransitRoute, TransitRouteStop, TransitSchedule, TransitStopFacility};
use std::collections::{HashMap};
use rust_qsim::simulation::id::Id;
use rust_qsim::simulation::time::SimTime;
use rand::seq::IteratorRandom;

pub struct TransitTrip {
    pub route: TransitRoute,
    pub departure: SimTime,
}
impl TransitTrip {
    pub fn from_route(route: TransitRoute, departure: SimTime) -> Self {
        TransitTrip {
            route,
            departure
        }
    }
}
fn get_earlier_stop(
    route: &TransitRoute,
    left: Id<TransitStopFacility>,
    right: Id<TransitStopFacility>
) -> Id<TransitStopFacility> {
    let stops = route.stops.clone();
    let l = stops.iter().position(|s| s.facility_id == left).unwrap();
    let r = stops.iter().position(|s| s.facility_id == right).unwrap();
    return if l < r {
        left
    } else {
        right
    }
}
fn get_stops_after<'a>(
    route: &'a TransitRoute,
    stop: &Id<TransitStopFacility>
) -> &'a [TransitRouteStop] {
    let pos = route
        .stops
        .iter()
        .position(|s| s.facility_id == *stop)
        .unwrap();

    &route.stops[pos..]
}

fn get_routes_serving_stop<'a>(
    stop: &Id<TransitStopFacility>,
    index: &'a HashMap<Id<TransitStopFacility>, Vec<&'a TransitRoute>>
) -> &'a [ &'a TransitRoute ] {
    index
        .get(stop)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

fn arrival_at_stop(trip: &TransitTrip, stop : &Id<TransitStopFacility>) -> Option<SimTime> {
    //TODO check, vibecoded
    trip.route.stops
        .iter()
        .find(|s| s.facility_id == stop.clone())
        .and_then(|s| {
            s.arrival_offset
                .map(|offset| trip.departure.saturating_add(offset))
        })
}

fn departure_at_stop(trip: &TransitTrip, stop : &Id<TransitStopFacility>) -> Option<SimTime> {
    //TODO check, vibecoded
    trip.route.stops
        .iter()
        .find(|s| s.facility_id == stop.clone())
        .and_then(|s| {
            s.departure_offset
                .map(|offset| trip.departure.saturating_add(offset))
        })
}
//TODO: Warning, vibecoded!
fn get_earliest_trip(route: &TransitRoute, stop: &Id<TransitStopFacility>, time: &SimTime) ->  Option<TransitTrip> {
    let stop_offset = route
        .stops
        .iter()
        .find(|s| s.facility_id == stop.clone())
        .and_then(|s| s.departure_offset)?;

    route
        .departures
        .iter()
        .find(|depart| {
            depart
                .departure_time
                .saturating_add(stop_offset)
                >= *time
        })
        .map(|depart| TransitTrip {
            route: route.clone(),
            departure: depart.departure_time,
        })
}
fn main() {
    println!("Loading schedule...");

    let path = Path::new("berlin-v7.1-10pct_transitSchedule.xml");
    let schedule = TransitSchedule::from_file(path);

    println!("Loading done.");

    let lines = schedule.lines();

    let facilities = schedule.facilities();



    //init




    let mut routes: Vec<&TransitRoute> = Vec::new();
    let mut routes_map: HashMap<Id<TransitRoute>,&TransitRoute> = HashMap::new();
    let footpaths = footpaths::build_footpath_graph(&facilities, 1000.0);

    //print_footpath_graph(&footpaths);
    for (l , li) in lines {
        for (r_id, r) in li.routes.iter() {
            routes.push(r);
            routes_map.insert(r_id.clone(), r);
        }

    }

    let mut stop_to_routes: HashMap<
        Id<TransitStopFacility>,
        Vec<&TransitRoute>
    > = HashMap::new();

    for route in routes.iter() {
        for stop in &route.stops {
            stop_to_routes
                .entry(stop.facility_id.clone())
                .or_default()
                .push(route);
        }
    }

    //foreach k ← 1, 2, . . . do

    let mut rng = rand::rng();
    use std::time::Instant;
    let now = Instant::now();
    for i in 1..100 {
        //
        println!("{}", i);
        let mut selected = facilities.keys().choose_multiple(&mut rng, 2);

        let start_facility = selected.pop().unwrap();
        let pt = selected.pop().unwrap();

        let mut marked_stops : Vec<Id<TransitStopFacility>> = Vec::new();
        marked_stops.push(start_facility.clone());

        let mut earliest_arrival: HashMap<Id<TransitStopFacility>, SimTime> = HashMap::new();

        for (f, _) in facilities.iter() {
            earliest_arrival.insert(f.clone(), SimTime::from_nanos(u64::MAX));
        }
        let mut earliest_arrival_last_round: HashMap<Id<TransitStopFacility>, SimTime>;

        if let Some(x) = earliest_arrival.get_mut(start_facility){
            *x = SimTime::from_nanos(0);
        }

        earliest_arrival_last_round = earliest_arrival.clone();

        let mut q: HashMap<Id<TransitRoute>,Id<TransitStopFacility>> = HashMap::new();

        for k in 1..5 {
            // Accumulate routes serving marked stops from previous round
            // 7 Clear Q
            q.clear();

            // 8 foreach marked stop p do
            for p in marked_stops.iter() {
                // 9 foreach route r serving p do
                for r in get_routes_serving_stop(p, &stop_to_routes) {
                    //10 if (r, p′) ∈ Q for some stop p′ then and 12 else 13 Add (r, p) to Q                    // 11 Substitute (r, p′) by (r, p) in Q if p comes before p′ in r
                    let p_dash = q.entry(r.id.clone()).or_insert(p.clone());

                    //11 Substitute (r, p′) by (r, p) in Q if p comes before p′ in r
                    *p_dash = get_earlier_stop(r, p.clone(), p_dash.clone());
                }
            }
            //14 unmark p
            marked_stops.clear();
            // Traverse each route
            // 15 foreach route (r, p) ∈ Q do
            for (r, p) in q.iter() {
                //16 t ← ⊥ // the current trip
                let mut trip: Option<TransitTrip> = None;

                //17 foreach stop pi of r beginning with p do
                for pi in get_stops_after(routes_map.get(r).unwrap(), p) {
                    // Can the label be improved in this round? Includes local and target pruning
                    //18 if t =/= ⊥ and arr(t, pi) < min{τ ∗(pi), τ ∗(pt)} then
                    if let Some(tr) = &trip {
                        if let Some(arr) =
                            arrival_at_stop(tr, &pi.facility_id)
                        {
                            if arr < earliest_arrival[&pi.facility_id]
                                && arr < earliest_arrival[&pt]
                            {
                                earliest_arrival.insert(
                                    pi.facility_id.clone(),
                                    arr,
                                );

                                marked_stops.push(
                                    pi.facility_id.clone(),
                                );
                            }
                        }
                    }
                    // Can we catch an earlier trip at pi?
                    //22 if τk−1(pi) ≤ τdep(t, pi) then

                    let should_board =
                        trip
                            .as_ref()
                            .map(|tr| {
                                earliest_arrival_last_round[&pi.facility_id]
                                    <= departure_at_stop(tr, &pi.facility_id).unwrap()
                            })
                            .unwrap_or(true);

                    if should_board {
                        trip = get_earliest_trip(
                            routes_map.get(r).unwrap(),
                            &pi.facility_id,
                            earliest_arrival_last_round
                                .get(&pi.facility_id)
                                .unwrap(),
                        );
                    }
                }
            }

            let mut marked_stops_next_round: Vec<Id<TransitStopFacility>> = Vec::new();
            for p in marked_stops.iter() {
                if let Some(edges) = footpaths.get(p) {
                    if let Some(&tp) = earliest_arrival.get(p) {
                        for (p_dash, walk_time) in edges {
                            let candidate = tp.saturating_add(walk_time.as_duration());

                            let is_improvement = match earliest_arrival.get(p_dash) {
                                Some(&current_best) => candidate < current_best,
                                None => true,
                            };

                            if is_improvement {
                                earliest_arrival.insert(p_dash.clone(), candidate);

                                // 27 mark p′
                                marked_stops_next_round.push(p_dash.clone());
                            }
                        }
                    }
                }
            }
            marked_stops.extend(marked_stops_next_round);

            earliest_arrival_last_round = earliest_arrival.clone();
        }
    }
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}


