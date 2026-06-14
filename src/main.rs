use std::path::Path;
use rust_qsim::simulation::pt::{TransitLine, TransitRoute, TransitRouteStop, TransitSchedule, TransitStopFacility};
use std::collections::{BTreeMap, HashMap};
use std::iter::Map;
use std::time::Duration;
use rust_qsim::simulation::events::DynEq;
use rust_qsim::simulation::id::Id;
use rust_qsim::simulation::time::SimTime;


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
fn get_stops_after(route: TransitRoute, stop: Id<TransitStopFacility>) -> Vec<TransitRouteStop> {
    let stops = route.stops;
    let pos = stops.iter().position(|s| s.facility_id == stop).unwrap();
    stops[pos..].to_vec()
}

fn get_routes_serving_stop<'a>(stop: &Id<TransitStopFacility>, routes_map : &'a [TransitRoute]) -> Vec<&'a TransitRoute> {
    routes_map
        .iter()
        .filter(|route| route.stops.iter().any(|s| s.facility_id == *stop))
        .collect()
}

fn arrival_at_stop(trip: &TransitRoute, stop : &Id<TransitStopFacility>) -> Option<SimTime> {
    //TODO check, vibecoded
    trip.stops
        .iter()
        .find(|s| s.facility_id == stop.clone())
        .and_then(|s| {
            s.arrival_offset
                .map(SimTime::from_duration)
        })
}

fn departure_at_stop(trip: &TransitRoute, stop : &Id<TransitStopFacility>) -> Option<SimTime> {
    //TODO check, vibecoded
    trip.stops
        .iter()
        .find(|s| s.facility_id == stop.clone())
        .and_then(|s| {
            s.departure_offset
                .map(SimTime::from_duration)
        })
}

fn main() {
    println!("Loading schedule...");

    let path = Path::new("simple_transitSchedule.xml");
    let schedule = TransitSchedule::from_file(path);

    println!("Loading done.");


    let lines = schedule.lines(); //TODO: Why does this work? How can Id<Transitline> be an integer?
    for (l, li) in lines {
        println!("Line: {:?}", li.routes.iter().next().unwrap());

        println!("\n");
    }
    let facilities = schedule.facilities();

    let mut iter = facilities.iter();
    let (start_facility, _) = iter.next().unwrap();
    iter.next();
    let (pt, _) = iter.next().unwrap();

    let mut earliest_arrival: HashMap<Id<TransitStopFacility>, SimTime> = HashMap::new();
    let mut earliest_arrival_last_round: HashMap<Id<TransitStopFacility>, SimTime> = HashMap::new();

    //init

    for (f, _) in facilities.iter() {
        earliest_arrival.insert(f.clone(), SimTime::from_nanos(u64::MAX));
        earliest_arrival_last_round.insert(f.clone(), SimTime::from_nanos(u64::MAX));
    }

    if let Some(x) = earliest_arrival.get_mut(start_facility){
        *x = SimTime::from_nanos(0);
    }

    let mut marked_stops : Vec<Id<TransitStopFacility>> = Vec::new();
    marked_stops.push(start_facility.clone());


    let mut q: HashMap<Id<TransitRoute>,Id<TransitStopFacility>> = HashMap::new();

    let mut routes: Vec<TransitRoute> = Vec::new();
    let mut routes_map: HashMap<Id<TransitRoute>,TransitRoute> = HashMap::new();

    for (l , li) in lines {
        for (r_id, r) in li.routes.iter() {
            println!("Route: {:?}", r.stops);
            routes.push(r.clone());
            routes_map.insert(r_id.clone(), r.clone());
            println!("\n");

        }

    }


    //foreach k ← 1, 2, . . . do
    for k in 1..10 {
        // Accumulate routes serving marked stops from previous round
        // 7 Clear Q
        q.clear();

        // 8 foreach marked stop p do
        for p in marked_stops.iter() {
            // 9 foreach route r serving p do
            for r in get_routes_serving_stop(p, &routes){
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
        for (r , p) in q.iter() {
            //16 t ← ⊥ // the current trip
            let trip : Option<&TransitRoute> = routes_map.get(r);

            //17 foreach stop pi of r beginning with p do
            for pi in get_stops_after(routes_map.get(r).unwrap().clone(),p.clone()){
                // Can the label be improved in this round? Includes local and target pruning
                //18 if t =/= ⊥ and arr(t, pi) < min{τ ∗(pi), τ ∗(pt)} then
                if let Some(trip) = & trip{
                    if arrival_at_stop(&trip,&pi.facility_id) < std::cmp::min(earliest_arrival.get(&pi.facility_id).copied(),earliest_arrival.get(&pt).copied()){
                        //19 τk(pi) ← τarr(t, pi)
                        //20 τ ∗(pi) ← τarr(t, pi) (implicitly done)
                        earliest_arrival.insert(pi.facility_id.clone(), arrival_at_stop(&trip,&pi.facility_id).unwrap());
                        //21 mark pi
                        marked_stops.push(pi.facility_id.clone())
                    }
                }
                // Can we catch an earlier trip at pi?
                //22 if τk−1(pi) ≤ τdep(t, pi) then

                /*if let Some(trip) = & trip{
                    if(earliest_arrival.get(&pi.facility_id) < departure_at_stop(&trip,&pi.facility_id).as_ref()){
                        //23 t ← et(r, pi)
                        trip = earliest_trip(r,pi)
                    }
                }*/


            }

        }
    /*for (&route, &p) in q.iter() {
                let mut current_trip: Option<TransitRoute> = None;
                let mut boarding_stop = p;
                //for &pi in get_stops_after(route, p).iter() {
            }*/


    }

    println!("{:?}",earliest_arrival.get(&pt))
}


