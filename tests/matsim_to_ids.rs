use rust_raptor::*;

#[cfg(test)]
mod tests {
    use rust_raptor::{generate_network, load_schedule};

    fn test(path: &str){
        let schedule = load_schedule(path);
        let (network, facitity_to_id, route_to_id) = generate_network(&schedule);
        assert_eq!(network.routes.len(),schedule.num_routes());
        for (line_id, line) in schedule.lines(){
            for (route_id, route) in line.routes.iter(){
                println!("line {} {:?}", line_id, route);
            }
        }

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