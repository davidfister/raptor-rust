use std::path::Path;
use rust_qsim::simulation::pt::{TransitRoute, TransitSchedule, TransitStopFacility};
use std::collections::{HashMap};
use std::time::Duration;
use rust_qsim::simulation::id::Id;




//TODO: For every stop, precompute routes serving this stop and the INDEX of the stop in this route
pub type RaptorRouteId = usize;
pub type RaptorStopId = usize;

pub type RaptorTripId = usize;
pub type RaptorTime = Duration; //TODO?

pub struct RaptorTrip {
    pub route: RaptorRouteId,
    pub arrivals: Vec<RaptorTime>,
    pub departures: Vec<RaptorTime>
}

pub struct RouteAtStop {
    route: RaptorRouteId,
    stop_id: RaptorStopId,
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

    for (line_id, line) in schedule.lines(){
        for (route_id, route) in line.routes.iter(){
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

    for (line_id, line) in schedule.lines(){
        for (route_id, route) in line.routes.iter(){
            let raptor_route_id = raptor_routes.len();

            let mut route_stops : Vec<RaptorStopId> = Vec::new();
            let mut route_trips : Vec<RaptorTripId> = Vec::new(); //TODO test with preallocation

            for stop in route.stops.iter(){
                route_stops.push(stop_ids[&stop.facility_id]);
                raptor_routes_at_stop[stop_ids[&stop.facility_id]].push(
                    RouteAtStop{
                        route: raptor_route_id,
                        stop_id: 0}); //TODO set right Stop ID, it does not have to be stop id but index of stop in the route array

            }
            //TODO move trips into route?
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


pub fn raptor(network: &RaptorNetwork, start : RaptorStopId, destination: RaptorStopId, rounds : usize) -> Duration {
    let mut earliest_arrival : Vec<Duration> = (0..network.stop_ids.len())
        .map(|_| Duration::MAX)
        .collect(); //TODO best way to do it?

    earliest_arrival[start] = Duration::ZERO;

    let mut marked_stops : Vec<RaptorStopId> = Vec::new();
    marked_stops.push(start);

    let mut Q : HashMap<RaptorRouteId,RaptorStopId> = HashMap::new();

    for k in 0.. rounds{
        Q.clear();

        for p in &marked_stops{
            for r in &network.routes_at_stop[*p]{
                Q
                    .entry(r.route)
                    .and_modify(|p_dash| *p_dash = earlier_stop(p, p_dash, &network.routes[r.route]))
                    .or_insert(*p);
            }
        }
        marked_stops.clear();

        for (r, p) in Q.iter(){
            let mut t : Option<RaptorTripId> = None;

            for p_i in network.routes[*r].stops[get_stop_index(&network.routes[*r], *p)..].iter() {
                if t != None && arrival_at_stop(&network.trips[t.unwrap()],&network.routes[*r], *p) < earliest_arrival[*p_i]{
                    earliest_arrival[*p_i] = arrival_at_stop(&network.trips[t.unwrap()],&network.routes[*r], *p);
                    marked_stops.push(*p_i);
                }
                if t != None && earliest_arrival[*p_i] < departure_at_stop(&network.trips[t.unwrap()],&network.routes[*r], *p){
                    t = Option::from(earliest_catchable_trip_at_stop(&network.trips, &network.routes[*r], *p, &earliest_arrival));
                }
                else if t == None{
                    t = Option::from(earliest_catchable_trip_at_stop(&network.trips, &network.routes[*r], *p, &earliest_arrival));
                }
            }
        }
    }
    earliest_arrival[destination]
}

fn earliest_catchable_trip_at_stop(trips : &Vec<RaptorTrip>, route : &RaptorRoute, stop : RaptorStopId, earliest_arrival : &Vec<Duration>) -> Option<RaptorTripId> {
    let stops = &route.stops;
    let p = stops.iter().position(|&s| s == stop).unwrap(); //TODO: faster way to do this?
    trips.iter().position(|t| t.departures[p] <= earliest_arrival[p]) //TODO: this can be done better
}

fn arrival_at_stop(trip : &RaptorTrip, route : &RaptorRoute, stop : RaptorStopId) -> Duration {
    let stops = &route.stops;
    let p = stops.iter().position(|&s| s == stop).unwrap(); //TODO: faster way to do this?
    trip.arrivals[p]
}

fn departure_at_stop(trip : &RaptorTrip, route : &RaptorRoute, stop : RaptorStopId) -> Duration {
    let stops = &route.stops;
    let p = stops.iter().position(|&s| s == stop).unwrap(); //TODO: faster way to do this?
    trip.departures[p]
}

fn earlier_stop(s0: &RaptorStopId, s1: &RaptorStopId, route : &RaptorRoute) -> RaptorStopId { //TODO: in function, use & or not for RaptorStopId etc?
    let stops = &route.stops;
    let l = stops.iter().position(|&s| s == *s0).unwrap();
    let r = stops.iter().position(|&s| s == *s1).unwrap();
    stops[l.min(r)]
}

fn get_stop_index(route : &RaptorRoute, stop : RaptorStopId) -> usize {
    let stops = &route.stops;
    stops.iter().position(|&s| s == stop).unwrap()
}


