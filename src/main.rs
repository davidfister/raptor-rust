use rust_raptor::*;

#[hotpath::main]
fn main() {
    println!("Loading schedule...");

    let schedule = load_schedule("data/berlin-v7.1-10pct_transitSchedule.xml");

    println!("Loading done.");

    //init
    let network = generate_network(&schedule);
    for i in 0..10{
        println!("{i}");
        let result =  raptor(&network,1,i+1,10);
    }





    /*for (i,r) in network.routes.iter().enumerate(){
        println!("Route: {i}");
        for s in network.routes[i].stops.iter(){
            println!("{s}");
        }
        for s in network.routes[i].trips.iter(){
            println!("{s}");
        }

    }
    for (i,t) in network.trips.iter().enumerate(){
        println!("Trip: {i}");
        let a = network.trips[i].route;
        println!("{a}");
        for t in network.trips[i].arrivals.iter(){
            println!("{t:?}");
        }
    }*/
}




