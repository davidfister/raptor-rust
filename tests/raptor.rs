use rust_raptor::*;

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use rust_raptor::{generate_network, load_schedule, raptor};
    fn test(path: &str){

    }
    #[test]
    fn test_simple(){
        let schedule = load_schedule("data/simple_transitSchedule.xml");
        let network = generate_network(&schedule);

        for (i, s) in network.stop_ids.iter(){
            println!("{i:?}, {s}")
        }

        //assert_eq!(raptor(&network,0,3,5), Duration::from_mins(2));
        assert_eq!(raptor(&network,0,1,5), Duration::from_mins(1));
        assert_eq!(raptor(&network,1,6,5), Duration::from_mins(60));


    }
    #[test]
    fn test_dresden() {
        test("oberlausitz-dresden-v2025.0-transitSchedule.xml")
    }
}