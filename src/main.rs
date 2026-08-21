mod footpaths;

use std::path::Path;
use rust_qsim::simulation::pt::{TransitDeparture, TransitLine, TransitRoute, TransitRouteStop, TransitSchedule, TransitStopFacility};
use std::collections::{HashMap};
use std::time::Duration;
use hotpath::Route;
use rust_qsim::simulation::id::Id;
use rust_qsim::simulation::time::SimTime;
use rand::seq::IteratorRandom;

//What do i need:
//Storing Arrival Times > Vec<SimTime>
//Q: Vec<(u32, u32)>
//Routes: Vec<Route>
//Route: {stops: Vec<u32>, trips: Vec<u32>}
//Trips: {route: u32, departures: Vec<SimTime>}
//I define Routes and Trips somewhere in main. Whenever i need the actual values, i do e.g. routes[id] and get back a reference(?) to a vector

//TODO: For every stop, precompute routes serving this stop and the INDEX of the stop in this route
type RaptorRouteId = usize;
type RaptorStopId = usize;

type RaptorTripId = usize;
type RaptorTime = Duration; //TODO?

struct RaptorTrip {
    pub route: RaptorRouteId,
    pub arrivals: Vec<RaptorTime>,
    pub departures: Vec<RaptorTime>
}

struct RouteAtStop {
    route: RaptorRouteId,
    stop_id: RaptorStopId,
}
struct RaptorNetwork {
    pub routes: Vec<RaptorRoute>,
    pub trips: Vec<RaptorTrip>,
    pub routes_at_stop: Vec<Vec<RouteAtStop>>
}

struct RaptorRoute{
    pub stops: Vec<RaptorStopId>,
    pub trips: Vec<RaptorTripId>
}

fn generate_network(schedule: &TransitSchedule) -> RaptorNetwork {
    let mut stop_ids : HashMap<Id<TransitStopFacility>,RaptorStopId> = HashMap::new();
    let mut route_ids : HashMap<Id<TransitRoute>,RaptorRouteId> = HashMap::new();

    for id in schedule.facilities().keys() {
        let raptor_id = stop_ids.len();
        stop_ids.insert(id.clone(), raptor_id);
    }

    for (line_id, line) in schedule.lines(){
        for (route_id, route) in line.routes.iter(){
            let raptor_id = route_ids.len();
            route_ids.insert(route_id.clone(),raptor_id);
        }
    }

    let mut raptor_routes : Vec<RaptorRoute> = Vec::new();
    let mut raptor_trips : Vec<RaptorTrip> = Vec::new();
    let mut raptor_routes_at_stop : Vec<Vec<RouteAtStop>> = Vec::with_capacity(schedule.facilities().len());

    for (line_id, line) in schedule.lines(){
        for (route_id, route) in line.routes.iter(){
            let raptor_route_id = raptor_routes.len();

            let mut route_stops : Vec<RaptorStopId> = Vec::new();
            let mut route_trips : Vec<RaptorTripId> = Vec::new(); //TODO test with preallocation

            for stop in route.stops.iter(){
                route_stops.push(stop_ids[&stop.facility_id]);
                raptor_routes_at_stop[stop_ids.len()-1].push(RouteAtStop{
                    route: raptor_route_id,
                    stop_id: 0}); //TODO set right Stop ID

            }
            for dep in route.departures.iter(){
                let mut raptor_arrivals : Vec<RaptorTime> = Vec::new();
                let mut raptor_departures : Vec<RaptorTime> = Vec::new();
                for stop in route.stops.iter(){
                    let arrival = stop.arrival_offset.unwrap() + dep.departure_time.as_duration();
                    raptor_arrivals.push(arrival);
                    let departure = stop.departure_offset.unwrap() + dep.departure_time.as_duration();
                    raptor_departures.push(departure);
                }
                route_trips.push(raptor_trips.len());
                raptor_trips.push(RaptorTrip {
                    route: raptor_route_id,
                    arrivals: raptor_arrivals,
                    departures: raptor_departures
                });
            }

            raptor_routes.insert(route_ids.get(route_id).unwrap().clone(),RaptorRoute{stops: route_stops, trips: route_trips}); //TODO change to push

        }

    }

    // for (id, newid) in stop_ids{
    //     println!("{id}, {newid}")
    // }
    // for (id, newid) in route_ids{
    //     println!("{id}, {newid}")
    // }

    RaptorNetwork {
        routes: raptor_routes,
        trips: raptor_trips,
        routes_at_stop: raptor_routes_at_stop
    }
}


#[hotpath::measure]
fn get_earlier_stop<'a>(
    route: &TransitRoute,
    left: &'a Id<TransitStopFacility>,
    right: &'a Id<TransitStopFacility>
) -> &'a Id<TransitStopFacility> {
    let l = route.stops.iter().position(|s| &s.facility_id == left).unwrap();
    let r = route.stops.iter().position(|s| &s.facility_id == right).unwrap();
    return if l < r {
        left
    } else {
        right
    }
}

#[hotpath::measure]
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
#[hotpath::measure]
fn get_routes_serving_stop<'a>(
    stop: &Id<TransitStopFacility>,
    index: &'a HashMap<Id<TransitStopFacility>, Vec<&'a TransitRoute>>
) -> &'a [ &'a TransitRoute ] {
    index
        .get(stop)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}
#[hotpath::measure]
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

#[hotpath::measure]
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
#[hotpath::measure]
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

#[hotpath::measure]
fn load_schedule(path_str: &str) -> TransitSchedule {
    let path = Path::new(path_str);
    TransitSchedule::from_file(path)
}

fn raptor(network: &RaptorNetwork, start : RaptorStopId, stop: RaptorStopId, rounds : usize) -> () {
    let mut q: HashMap<RaptorRouteId,RaptorStopId> = HashMap::new();
    let mut marked_stops : Vec<RaptorStopId> = Vec::new();
    let mut earliest_arrival: Vec<Duration> = Vec::new();
    //let mut earliest_arrival_last_round: Vec<Duration> = Vec::new();





    for k in 1..rounds {
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
                //p_dash = get_earlier_stop(r, &p, &p_dash);
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
}*/
}

#[hotpath::main]
fn main() {
    println!("Loading schedule...");

    let schedule = load_schedule("simple_transitSchedule.xml");

    println!("Loading done.");

    //init
    let network : RaptorNetwork = generate_network(&schedule);

    {// for (i,r) in network.routes.iter().enumerate(){
    //     println!("Route: {i}");
    //     for s in network.routes[i].stops.iter(){
    //         println!("{s}");
    //     }
    //     for s in network.routes[i].trips.iter(){
    //         println!("{s}");
    //     }
    //
    // }
    // for (i,t) in network.trips.iter().enumerate(){
    //     println!("Trip: {i}");
    //     let a = network.trips[i].route;
    //     println!("{a}");
    //     for t in network.trips[i].arrivals.iter(){
    //         println!("{t:?}");
    //     }
    //
    // }
    }

}




