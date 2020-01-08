extern crate toml;

use std::cell::RefCell;
use std::rc::Rc;
use std::fs;

pub struct Waypoint {
    pub coords: (f32, f32),
    pub children: RefCell<Vec<Rc<Waypoint>>>
}

impl Waypoint {
    fn new(co: (f32,f32)) -> Self {
        Waypoint { coords: co, children: RefCell::new(vec![]) }
    }
}

pub mod builder {
    use serde::{Serialize, Deserialize};
    use super::*;

    #[derive(Deserialize)]
    struct Template {
        // [id, (coord_x, coord_y), [dest_points...]]
        waypoints: Vec<(u32, (f32, f32), Vec<u32>)>,
    }

    fn template_to_waypoints(t: Template) -> Result<Vec<Waypoint>, &'static str> {
        let mut res: Vec<(u32, Waypoint)> = Vec::new();
        for way in t.waypoints {
            res.push((way.0, Waypoint::new(way.1)));
        }
        let res = res.into_iter().map(|v| v.1).collect();
        return Ok(res);
    }
}


#[cfg(test)]
mod tests {

    use super::*;
    use super::builder::Template;

    #[test]
    fn can_access_map_file(){
        let contents = fs::read_to_string("res/map1.toml").unwrap();
        let temp: Template =  toml::from_str(&contents).unwrap();
    }
}

