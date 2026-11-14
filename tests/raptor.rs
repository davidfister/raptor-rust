#[cfg(test)]
mod tests {
    use rust_raptor::{generate_network, load_schedule, raptor};
    #[test]
    fn test_berlin(){
        let schedule = load_schedule("data/berlin-v7.1-10pct_transitSchedule.xml");
        let network = generate_network(&schedule);
        //Plausibility tests
        //assert_eq!(raptor(&network, (793080.0, 5826004.0), (793544.0, 5829277.0), 1200, 10), 60);  //Savignyplatz to Beusselstr. (17-22 min)
        //assert_eq!(raptor(&network, (797627.0, 5831132.0), (801006.0, 5830363.0), 1200, 10), 60);  //Gesundbrunnen to Greifswalder Str. (6-11 min)
        //assert_eq!(raptor(&network, (793291.0, 5826779.0), (799232.0, 5828196.0), 1200, 10), 60);  //Ernst-Reuter to Alexanderplz. (22-32 min)
        assert_eq!(raptor(&network, (809407.0, 5834353.0), (784660.0, 5828789.0), 1200, 10), 60);  //Ahrensfelde to Spandau. (58-71 min)




    }
}