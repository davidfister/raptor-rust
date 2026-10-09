#[cfg(test)]
mod tests {
    use rust_raptor::{generate_network, load_schedule, raptor};
    #[test]
    fn test_simple() {
        let schedule = load_schedule("data/simple_transitSchedule.xml");
        let network = generate_network(&schedule);

        for (i, s) in network.stop_ids.iter() {
            println!("{i:?}, {s}")
        }


        assert_eq!(raptor(&network, 0, 1, 5), 60);
        assert_eq!(raptor(&network, 1, 2, 5), 3*60);
        assert_eq!(raptor(&network, 0, 2, 5), 3*60);
        assert_eq!(raptor(&network, 0, 6, 5), 90*60);

    }
}