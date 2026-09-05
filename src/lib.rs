use std::path::Path;
use std::collections::HashMap;
use std::time::Duration;
use std::usize;
use rust_qsim::simulation::pt::{TransitRoute, TransitSchedule, TransitStopFacility};
use rust_qsim::simulation::id::Id;




//TODO: For every stop, precompute routes serving this stop and the INDEX of the stop in this route
pub type RaptorRouteId = usize;
pub type RaptorStopId = usize;

pub type RaptorTripId = usize;
pub type RaptorTime = usize;

pub struct RaptorTrip {
    pub route: RaptorRouteId,
    pub arrivals: Vec<RaptorTime>,
    pub departures: Vec<RaptorTime>
}

struct RouteAtStop {
    route: RaptorRouteId,
    stop_idx: usize,
}
pub struct RaptorNetwork {
    pub routes: Vec<RaptorRoute>,
    pub trips: Vec<RaptorTrip>,
    pub routes_at_stop: Vec<Vec<RouteAtStop>>,
    pub stop_ids : HashMap<Id<TransitStopFacility>,RaptorStopId>,
    pub route_ids : HashMap<Id<TransitRoute>,RaptorRouteId>
}

pub struct RaptorRoute{
    pub stops: Vec<RaptorStopId>,
    pub trips: Vec<RaptorTripId>
}

pub fn generate_network(schedule: &TransitSchedule) -> RaptorNetwork {
    let mut stop_ids : HashMap<Id<TransitStopFacility>,RaptorStopId> = HashMap::new();
    let mut route_ids : HashMap<Id<TransitRoute>,RaptorRouteId> = HashMap::new();

    for id in schedule.facilities().keys() {
        let raptor_id = stop_ids.len();
        stop_ids.insert(id.clone(), raptor_id);
    }

    for (_, line) in schedule.lines(){
        for (route_id, _) in line.routes.iter(){
            let raptor_id = route_ids.len();
            route_ids.insert(route_id.clone(),raptor_id);
        }
    }

    let mut raptor_routes : Vec<RaptorRoute> = Vec::new();
    let mut raptor_trips : Vec<RaptorTrip> = Vec::new();
    let mut raptor_routes_at_stop : Vec<Vec<RouteAtStop>> =
        (0..schedule.facilities().len())
            .map(|_| Vec::new())
            .collect();

    for (_, line) in schedule.lines(){
        for (_, route) in line.routes.iter(){
            let raptor_route_id = raptor_routes.len();

            let mut route_stops : Vec<RaptorStopId> = Vec::new();
            let mut route_trips : Vec<RaptorTripId> = Vec::new(); //TODO test with preallocation

            for (idx, stop) in route.stops.iter().enumerate(){
                let stop_id = stop_ids[&stop.facility_id];
                route_stops.push(stop_id);
                raptor_routes_at_stop[stop_id].push(
                    RouteAtStop{
                        route: raptor_route_id,
                        stop_idx: idx});
            }
            //TODO move trips into route?
            for dep in route.departures.iter(){
                let mut raptor_arrivals : Vec<RaptorTime> = Vec::new();
                let mut raptor_departures : Vec<RaptorTime> = Vec::new();
                for stop in route.stops.iter(){
                    let arrival = stop.arrival_offset.unwrap().as_secs() + dep.departure_time.as_secs();
                    raptor_arrivals.push(arrival as RaptorTime);
                    let departure = stop.departure_offset.unwrap().as_secs() + dep.departure_time.as_secs();
                    raptor_departures.push(departure as RaptorTime);
                }
                route_trips.push(raptor_trips.len());
                raptor_trips.push(RaptorTrip {
                    route: raptor_route_id,
                    arrivals: raptor_arrivals,
                    departures: raptor_departures
                });
            }

            route_trips.sort_by_key(|&t| raptor_trips[t].departures[0]);
            raptor_routes.push(RaptorRoute{stops: route_stops, trips: route_trips});

        }

    }

    RaptorNetwork {
        routes: raptor_routes,
        trips: raptor_trips,
        routes_at_stop: raptor_routes_at_stop,
        stop_ids: stop_ids,
        route_ids: route_ids
    }
}

#[hotpath::measure]
pub fn load_schedule(path_str: &str) -> TransitSchedule {
    let path = Path::new(path_str);
    TransitSchedule::from_file(path)
}


#[hotpath::measure]
pub fn raptor(
    network: &RaptorNetwork,
    start: RaptorStopId,
    destination: RaptorStopId,
    rounds : usize
) -> usize {
    let mut earliest_arrival : Vec<RaptorTime> = vec![usize::MAX; network.stop_ids.len()];

    earliest_arrival[start] = 0;

    let mut marked_stops : Vec<RaptorStopId> = vec![start];

    let mut q : HashMap<RaptorRouteId, usize> = HashMap::new(); //Storing Route and Route based index
    let mut earliest_arrival_last_round = Vec::with_capacity(network.stop_ids.len());

    for _ in 0.. rounds{
        earliest_arrival_last_round.clone_from(&earliest_arrival);
        q.clear();

        for p in &marked_stops{
            for r in &network.routes_at_stop[*p]{
                q
                    .entry(r.route)
                    .and_modify(|p_dash| *p_dash = (*p_dash).min(r.stop_idx))
                    .or_insert(r.stop_idx);
            }
        }
        marked_stops.clear();

        for (q_route_id, q_stop_index) in &q{
            let mut t : Option<RaptorTripId> = None;

            let route = &network.routes[*q_route_id];

            for (stop_index, stop_id) in route.stops.iter().enumerate().skip(*q_stop_index) {
                if let Some(trip_id) = t{
                    let trip = &network.trips[trip_id];
                    let arrival = arrival_at_stop(trip,route, stop_index);

                    if arrival < earliest_arrival[destination] && arrival < earliest_arrival[*stop_id]{
                        marked_stops.push(*stop_id);
                        earliest_arrival[*stop_id] = arrival;
                    }
                    if earliest_arrival_last_round[*stop_id] <= departure_at_stop(trip, route, stop_index) {
                        t = earliest_catchable_trip_at_stop(&network, route, *stop_id, stop_index, &earliest_arrival_last_round);
                    }
                }
                else {
                    t = earliest_catchable_trip_at_stop(&network, route, *stop_id, stop_index, &earliest_arrival_last_round); //TODO check correctness
                }
            }
        }
    }
    earliest_arrival[destination]
}

#[hotpath::measure]
fn earliest_catchable_trip_at_stop(network: &RaptorNetwork, route : &RaptorRoute, stop : RaptorStopId, stop_index : usize, earliest_arrival : &[usize], ) -> Option<RaptorTripId> { //TODO cleanup
    route.trips.iter().copied().find(|&t| network.trips[t].departures[stop_index] >= earliest_arrival[stop])
}

#[hotpath::measure]
fn arrival_at_stop(trip : &RaptorTrip, route : &RaptorRoute, stop_index : usize) -> usize {
    trip.arrivals[stop_index]
}

#[hotpath::measure]
fn departure_at_stop(trip : &RaptorTrip, route : &RaptorRoute, stop_index : usize) -> usize {
    trip.departures[stop_index]
}

#[hotpath::measure]
fn earlier_stop(s0: &RaptorStopId, s1: &RaptorStopId, route : &RaptorRoute) -> RaptorStopId { //TODO: in function, use & or not for RaptorStopId etc?
    let stops = &route.stops;
    let l = stops.iter().position(|&s| s == *s0).unwrap();
    let r = stops.iter().position(|&s| s == *s1).unwrap();
    stops[l.min(r)]
}

#[hotpath::measure]
fn get_stop_index(route : &RaptorRoute, stop : RaptorStopId) -> usize {
    let stops = &route.stops;
    stops.iter().position(|&s| s == stop).unwrap()
}
#[cfg(test)]
mod tests {
    use std::time::Duration;
    use crate::{earliest_catchable_trip_at_stop, generate_network, load_schedule};

    //#[test]
    fn test_earliest_trip(){
        let schedule = load_schedule("simple_transitSchedule.xml");
        let network = generate_network(&schedule);
        let mut earliest_arrival : Vec<Duration> = (0..network.stop_ids.len())
            .map(|_| Duration::MAX)
            .collect(); //TODO best way to do it?
        println!("{:?}", &network.routes[0].stops);
        println!("{:?}", &network.route_ids);

        for (_,line) in schedule.lines(){
            for (route_id, route) in line.routes.iter(){
                println!("{:?}", route);
                println!("{:?}", &network.route_ids[&route.id]);
            }
        }

        //assert_eq!(earliest_catchable_trip_at_stop(&network, &network.routes[0], 0, &earliest_arrival), None);
        earliest_arrival[0] = Duration::ZERO;
        //assert!(!earliest_catchable_trip_at_stop(&network, &network.routes[0], 0, &earliest_arrival).is_none());
    }

}
