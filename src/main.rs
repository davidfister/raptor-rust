use rust_raptor::*;

#[hotpath::main]
fn main() {
    println!("Loading schedule...");

    let schedule = load_schedule("data/simple_transitSchedule.xml");

    println!("Loading done.");

    //init
    let network = generate_network(&schedule);
    let d = raptor(&network,0,3,5);
    let t = d.as_secs()/60;
    println!("{t}")


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




