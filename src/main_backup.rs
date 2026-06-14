/*

mod main_paper;

use std::path::Path;
use rust_qsim::simulation::pt::{TransitLine, TransitRoute, TransitSchedule, TransitStopFacility};
use std::collections::HashMap;
use rust_qsim::simulation::id::Id;

fn main() {
    println!("Loading schedule...");

    let path = Path::new("/home/david/Sync/Studium/BA/Code/parallel-qsim-berlin/assets/dresden-v1.0-transitSchedule.xml.gz");
    let schedule = TransitSchedule::from_file(path);

    println!("Loading done.");


    let lines = schedule.lines(); //TODO: Why does this work? How can Id<Transitline> be an integer?
    for (l , li) in lines {
        println!("Line: {:?}", li.routes.iter().next().unwrap());
        println!("\n");
    }
    let facilities = schedule.facilities();

    let (start_facility, _) =facilities.iter().next().unwrap();


    let mut earliest_arrival : HashMap<Id<TransitStopFacility>,u64> = HashMap::new(); //TODO: In which format is time defined?
    let mut marked_stops : Vec<Id<TransitStopFacility>> = Vec::new();


    //Für jede Haltestelle v:
    //  earliestArrival[v] = ∞
    for (facility_id,_f) in facilities.iter(){
        earliest_arrival.insert(facility_id.clone(),u64::MAX);
    }

    //earliestArrival[S] = Startzeit
    if let Some(x) = earliest_arrival.get_mut(start_facility) {
        *x = 0;
    }

    //   markedStops = {S}
    marked_stops.push(start_facility.clone());


    //newMarkedStops = ∅
    let mut new_marked_stops : Vec<Id<TransitStopFacility>> = Vec::new();

    let mut routes: Vec<TransitRoute> = Vec::new();

    for (l , li) in lines {
        for (r_id, r) in li.routes.iter() {
            println!("Route: {:?}", r.stops);
            routes.push(r.clone())
        }

    }

    for i in 1..11{
        for r in routes.iter(){
            let mut contains_stop = false;
            for s in r.stops.iter(){
                if(marked_stops.contains(&s.facility_id)){
                    contains_stop = true;
                    break;
                }
            }
            if(contains_stop){
                let mut earliest_trip = 1;
            }

        }
    }

    //       Für jede Route r in Routen:
    //           Wenn r mindestens eine Haltestelle aus markedStops enthält:
    //               earliestTrip = frühester Trip auf r nach Ankunftszeiten der markierten Haltestellen
    //               Für jede Haltestelle u auf r nach earliestTrip:
    //                   arrivalTime = Ankunftszeit von earliestTrip an u
    //                   Wenn arrivalTime < earliestArrival[u]:
    //                       earliestArrival[u] = arrivalTime
    //                       newMarkedStops.add(u)


    //markedStops = newMarkedStops
    //       Wenn markedStops leer ist:
    //           Stoppen (keine Verbesserungen mehr möglich)


}


 */