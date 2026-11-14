use std::path::Path;
use std::collections::HashMap;
use rust_qsim::generated::general::Coordinate;
use rust_qsim::simulation::pt::{TransitRoute, TransitSchedule, TransitStopFacility};
use rust_qsim::simulation::id::Id;




//TODO: For every stop, precompute routes serving this stop and the INDEX of the stop in this route
pub type RaptorRouteId = usize;
pub type RaptorStopId = usize;

pub type RaptorTripId = usize;
pub type RaptorTime = usize;

pub struct RaptorTrip {
    pub arrivals: Vec<RaptorTime>,
    pub departures: Vec<RaptorTime>
}

pub struct RouteAtStop {
    route: RaptorRouteId,
    stop_idx: usize,
}
pub struct RaptorNetwork {
    pub routes: Vec<RaptorRoute>,
    pub trips: Vec<RaptorTrip>,
    pub routes_at_stop: Vec<Vec<RouteAtStop>>,
    pub stop_ids : HashMap<Id<TransitStopFacility>,RaptorStopId>,
    pub route_ids : HashMap<Id<TransitRoute>,RaptorRouteId>,
    pub footpaths: Vec<Vec<RaptorFootpath>>,
    pub grid : HashMap<(i32, i32), Vec<RaptorStopId>>,
    pub stops_x : Vec<f64>,
    pub stops_y : Vec<f64>,
}

pub struct RaptorRoute{
    pub stops: Vec<RaptorStopId>,
    pub trips: Vec<RaptorTripId>
}

pub struct RaptorFootpath {
    pub to: RaptorStopId,
    pub duration: RaptorTime,
}


pub fn generate_network(schedule: &TransitSchedule) -> RaptorNetwork {
    let mut stop_ids : HashMap<Id<TransitStopFacility>,RaptorStopId> = HashMap::new();
    let mut route_ids : HashMap<Id<TransitRoute>,RaptorRouteId> = HashMap::new();

    let mut coordinates : Vec<Coordinate> = Vec::new();
    let mut stops_x : Vec<f64> = Vec::new();
    let mut stops_y : Vec<f64> = Vec::new();


    for (id, facility) in schedule.facilities().iter() {
        let raptor_id = stop_ids.len();
        coordinates.push(Coordinate::from(facility.coord.clone()));
        stop_ids.insert(id.clone(), raptor_id);
        stops_x.push(facility.coord.x);
        stops_y.push(facility.coord.y); //TODO: coords get stored twice, simplify
    }

    for (_, line) in schedule.lines(){
        for (route_id, _) in line.routes.iter(){
            let raptor_id = route_ids.len();
            route_ids.insert(route_id.clone(),raptor_id);
            println!("{}", route_id);
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
                    let arrival = stop.arrival_offset.unwrap().as_secs() + dep.departure_time.as_secs(); //crash if arrival_offset is not set
                    raptor_arrivals.push(arrival as RaptorTime);
                    let departure = stop.departure_offset.unwrap().as_secs() + dep.departure_time.as_secs();
                    raptor_departures.push(departure as RaptorTime);
                }
                route_trips.push(raptor_trips.len());
                raptor_trips.push(RaptorTrip {
                    arrivals: raptor_arrivals,
                    departures: raptor_departures
                });
            }

            route_trips.sort_by_key(|&t| raptor_trips[t].departures[0]);
            raptor_routes.push(RaptorRoute{stops: route_stops, trips: route_trips});

        }

    }
    let mut footpaths  : Vec<Vec<RaptorFootpath>>  =
        (0..schedule.facilities().len())
            .map(|_| Vec::new())
            .collect();

    let mut grid: HashMap<(i32, i32), Vec<RaptorStopId>> = HashMap::new();

    const MAX_WALK_DISTANCE: f64 = 500.0;
    fn cell(x: f64, y: f64) -> (i32, i32) {
        (
            (x / MAX_WALK_DISTANCE).floor() as i32,
            (y / MAX_WALK_DISTANCE).floor() as i32,
        )
    }

    for stop_id in stop_ids.values() {
        let c = cell(coordinates[*stop_id].x, coordinates[*stop_id].y);
        grid.entry(c).or_default().push(*stop_id);
    }

    let max_distance_sq = MAX_WALK_DISTANCE * MAX_WALK_DISTANCE;

    for (_,stop_id) in stop_ids.values().enumerate() {
        let (cx, cy) = cell(coordinates[*stop_id].x, coordinates[*stop_id].y);

        for dx_cell in -1..=1 {
            for dy_cell in -1..=1 {
                let candidates = match grid.get(&(cx + dx_cell, cy + dy_cell)) {
                    Some(x) => x,
                    None => continue,
                };
                for &inner_id in candidates {
                    if *stop_id == inner_id {
                        continue;
                    }
                    let dx = coordinates[*stop_id].x - coordinates[inner_id].x;
                    let dy = coordinates[*stop_id].y - coordinates[inner_id].y;

                    let dist_sq = dx * dx + dy * dy;

                    if dist_sq <= max_distance_sq {
                        let duration = dist_sq.sqrt() / 0.83; //3 kmh, 0.83meter/sec TODO: put in const

                        footpaths[*stop_id].push(
                            RaptorFootpath {
                                to: inner_id,
                                duration: duration as usize,
                            }
                        );
                    }
                }
            }
        }
    }

    RaptorNetwork {
        routes: raptor_routes,
        trips: raptor_trips,
        routes_at_stop: raptor_routes_at_stop,
        stop_ids,
        route_ids,
        footpaths,
        grid,
        stops_x,
        stops_y
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
    start: (f64, f64),
    destination: (f64, f64),
    departure_time: RaptorTime,
    rounds : usize
) -> usize {

    //TODO: duplicate, move outside
    const MAX_WALK_DISTANCE: f64 = 500.0;
    fn cell(x: f64, y: f64) -> (i32, i32) {
        (
            (x / MAX_WALK_DISTANCE).floor() as i32,
            (y / MAX_WALK_DISTANCE).floor() as i32,
        )
    }

    // ----- 1. INIT PHASE -----
    let mut earliest_arrival : Vec<RaptorTime> = vec![usize::MAX;network.stop_ids.len()];
    let mut earliest_arrival_last_round = vec![usize::MAX;network.stop_ids.len()];

    let mut marked_stops : Vec<RaptorStopId> = Vec::new();
    let mut is_marked = vec![false; network.stop_ids.len()];

    let mut q_raptor_route_ids: Vec<RaptorRouteId> = Vec::new();
    let mut q_route_based_index: Vec<usize> = vec![usize::MAX; network.routes.len()]; //Storing Route and Route based index


    let mut round = 0;
    let mut max_rounds = rounds;

    // ----- initialize first stops -----
    let (cx, cy) = cell(start.0, start.1);
    let max_distance_sq = MAX_WALK_DISTANCE * MAX_WALK_DISTANCE;

    //TODO: duplicate?
    for dx_cell in -1..=1 {
        for dy_cell in -1..=1 {
            let stops_in_range = match network.grid.get(&(cx + dx_cell, cy + dy_cell)) {
                Some(x) => x,
                None => continue,
            };
            for &stop in stops_in_range {
                let dx = &network.stops_x[stop] - start.0;
                let dy = &network.stops_y[stop] - start.1;

                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= max_distance_sq {
                    marked_stops.push(stop);
                    is_marked[stop] = true;
                    earliest_arrival[stop] = departure_time + (dist_sq.sqrt() / 0.83) as usize;
                }
            }
        }
    }



    let (cx,cy) = cell(destination.0, destination.1);
    let max_distance_sq = MAX_WALK_DISTANCE * MAX_WALK_DISTANCE;

    let mut stops_in_dest_range = Vec::new();
    //TODO: duplicate?
    for dx_cell in -1..=1 {
        for dy_cell in -1..=1 {
            let stops_in_range = match network.grid.get(&(cx + dx_cell, cy + dy_cell)) {
                Some(x) => x,
                None => continue,
            };
            for &stop in stops_in_range {
                let dx = &network.stops_x[stop] - destination.0;
                let dy = &network.stops_y[stop] - destination.1;

                let dist_sq = dx * dx + dy * dy;

                if dist_sq <= max_distance_sq {
                    stops_in_dest_range.push(stop);
                }
            }
        }
    }

    let mut earliest_arrival_destination = usize::MAX;

    // ----- LOOP ENTER -----
    while round < max_rounds{
        round += 1;
        if marked_stops.is_empty() {
            break;
        }
        for &s in &marked_stops {
            earliest_arrival_last_round[s] = earliest_arrival[s];
        }
        q_raptor_route_ids.clear();
        q_route_based_index.fill(usize::MAX);

        //Disable heuristic
        /*if earliest_arrival_destination != usize::MAX {
            max_rounds = max_rounds.min(round + 3);
        }*/

        // ----- 2. STOPS TO ROUTES -----
        for p in &marked_stops{
            for r in &network.routes_at_stop[*p]{ //r is type RouteAtStop
                if q_route_based_index[r.route] == usize::MAX {
                    q_raptor_route_ids.push(r.route);
                }
                q_route_based_index[r.route] = q_route_based_index[r.route].min(r.stop_idx);
            }
        }

        for &p in &marked_stops { is_marked[p] = false; } //prepare marked_stops for next(this) round
        marked_stops.clear();

        // ----- 3. ROUTE ITERATION -----
        // - iterate over routes
        // - for all stops after the "hop on stop" check if they can be improved
        // - mark stops if they are reachable by footpaths and update arrival times
        for q_route_id in q_raptor_route_ids.iter(){
            let mut t : Option<RaptorTripId> = None;

            let route = &network.routes[*q_route_id];

            let q_stop_index = q_route_based_index[*q_route_id];
            for (stop_index, stop_id) in route.stops.iter().enumerate().skip(q_stop_index) {
                if let Some(trip_id) = t{
                    let trip = &network.trips[trip_id];
                    let arrival = arrival_at_stop(trip, route, stop_index);

                    if arrival < earliest_arrival_destination && arrival < earliest_arrival[*stop_id]{
                        if !is_marked[*stop_id] {
                            is_marked[*stop_id] = true;
                            marked_stops.push(*stop_id);
                        }
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
        let mut temp_v :  Vec<RaptorStopId> = Vec::new();


        let times_before_footpath = earliest_arrival.clone();

        for m in marked_stops.iter(){
            for footpath in &network.footpaths[*m] {
                let new_arrival = times_before_footpath[*m] + footpath.duration;

                if new_arrival < earliest_arrival[footpath.to] && new_arrival < earliest_arrival_destination {
                    earliest_arrival[footpath.to] = new_arrival;

                    if !is_marked[footpath.to] {
                        is_marked[footpath.to] = true;
                        temp_v.push(footpath.to);
                    }
                    

                }
            }
        }
        marked_stops.append(&mut temp_v);
        for s in &stops_in_dest_range{
            let dx = &network.stops_x[*s] - destination.0;
            let dy = &network.stops_y[*s] - destination.1;
            let walk_time = (dx * dx + dy * dy).sqrt() / 0.83;

            if earliest_arrival[*s] != usize::MAX {
                earliest_arrival_destination = earliest_arrival_destination.min(earliest_arrival[*s] + walk_time as RaptorTime);
            }
        }
    }
    earliest_arrival_destination
}

fn earliest_catchable_trip_at_stop(network: &RaptorNetwork, route : &RaptorRoute, stop : RaptorStopId, stop_index : usize, earliest_arrival : &[usize], ) -> Option<RaptorTripId> { //TODO cleanup
    route.trips.iter().copied().find(|&t| network.trips[t].departures[stop_index] >= earliest_arrival[stop])
}

fn arrival_at_stop(trip : &RaptorTrip, _route : &RaptorRoute, stop_index : usize) -> usize { //TODO remove Route
    trip.arrivals[stop_index]
}

fn departure_at_stop(trip : &RaptorTrip, _route : &RaptorRoute, stop_index : usize) -> usize {
    trip.departures[stop_index]
}