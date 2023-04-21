# Calendar library for Building Simulation

 This is a library containing nn extremely simple Date object. The
 purpose is to help perform Building Performance calculations, so it only
 contains month, day and hour (in decimals). **It does not consider years at all**, 
 days and Months are counted from 1 (e.g. January is 1, not 0). 
 We don't need anything else, I think.
 
 [CHECK THE DOCS!](https://simple-buildingsimulation.github.io/calendar/)
 
 # Interaction with Serde
 
 You can enable the `serde` feature and do stuff like this:
 
 ```rust
 use calendar::Date;
 use serde_json; // import "serde_json" and enable feature "serde"
 
 let v = r#"{"month": 9,"day": 4, "hour": 21}"#;
 let d : Date = serde_json::from_str(&v).unwrap();
 assert_eq!(d.month, 9);
 assert_eq!(d.day, 4);
 assert!((d.hour - 21.).abs() < 1e-5);
 ```
 
 # Interaction with Chrono
 
 You can enable the `chrono` feature and do stuff like this
 
 ```rust
 use chrono::NaiveDateTime; // enable feature "chrono"
 use calendar::Date;
        
 let v = "2014-11-28T21:00:09+09:00";
 let chrono_datetime  = NaiveDateTime::parse_from_str(&v, "%Y-%m-%dT%H:%M:%S%z").unwrap();
 
 let d : Date = chrono_datetime.into();
 assert_eq!(d.month, 11);
 assert_eq!(d.day, 28);
 assert!((d.hour - 21.0025).abs() < 1e-5, "hour is {}", d.hour);
 ```