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
    use std::collections::HashMap;

    #[derive(Deserialize)]
    struct Template {
        // [id, (coord_x, coord_y), [dest_points...]]
        waypoints: Vec<(u32, (f32, f32), Vec<u32>)>,
    }

    #[derive(Debug)]
    enum BuilderError {
        SelfReference(u32),
        UnknownIdForChild(u32, u32),
        RepeatedPoint(u32),
    }

    fn template_to_waypoints(t: Template) -> Result<Vec<Rc<Waypoint>>, BuilderError> {
        use BuilderError::*;
        let mut res = HashMap::new();
        for way in &t.waypoints {
            if res.contains_key(&way.0) { return Err(RepeatedPoint(way.0)); }
            res.insert(way.0, Rc::new(Waypoint::new(way.1)));
        }
        for way in t.waypoints {
            let mut current_children = res.get(&way.0).unwrap().children.borrow_mut();
            for id in way.2.iter() {
                if way.0 == *id { return Err(SelfReference(*id)); }
                current_children.push(Rc::clone(res.get(id).ok_or(UnknownIdForChild(way.0, *id))?));
            }
        }
        let res = res.into_iter().map(|v| v.1).collect();
        return Ok(res);
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn template_to_waypoints_basic() {
            let test_template = Template {
                waypoints: vec![
                    (1, (0.1, 0.2), vec![2,3]),
                    (2, (0.1, 0.2), vec![1,3]),
                    (3, (0.1, 0.2), vec![1]),
                ]
            };
            let result = template_to_waypoints(test_template).unwrap();
            assert!(result[0].children.borrow().len() > 0);
        }
    }
}
