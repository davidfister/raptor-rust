use rust_raptor::*;

#[cfg(test)]
mod tests {
    use rust_raptor::{generate_network, load_schedule};

    fn test(path: &str){
        let schedule = load_schedule(path);
        let network = generate_network(&schedule);
        assert_eq!(network.routes.len(),schedule.num_routes());
        // for i in network.stop_ids{
        //     println!("{i:?}")
        // }

        //Check that every RaptorStopId corresponds to the right stop of the Schedule
        for (line_id, line) in schedule.lines(){
            for (route_id, route) in line.routes.iter(){
                let a = &network.routes[network.route_ids[route_id]];
                let b = &a.stops;
                for (i,s) in route.stops.iter().enumerate(){
                    let c = b[i];
                    assert_eq!(network.stop_ids[&s.facility_id],c);
                }
            }
        }
        // for i in network.trips{
        //     let a = i.route;
        //     println!("{a:?}")
        // }

    }
    #[test]
    fn test_simple(){
        test("simple_transitSchedule.xml")

    }
    #[test]
    fn test_dresden() {
        test("oberlausitz-dresden-v2025.0-transitSchedule.xml")
    }
}