use std::cell::RefCell;
use std::rc::Rc;

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
    use super::*;
    use std::collections::HashMap;

    #[derive(Debug)]
    pub enum BuilderError {
        SelfReference (u32),
        UnknownIdForChild (u32, u32),
        RepeatedPoint (u32),
        IOError (std::io::Error),
        CouldNotReadLine (usize, &'static str)
    }

    pub fn open_map(path: &str) -> Result<Vec<Rc<Waypoint>>, BuilderError> {
        let temp = path_to_template(path)?;
        return  template_to_waypoints(temp);
    }

    fn path_to_template(path: &str) -> Result<Vec<(u32, (f32, f32), Vec<u32>)>, BuilderError> {
        use std::fs;
        use BuilderError::*;

        fn find_or_err(id: usize, wher: &str, what: &'static str) -> Result<usize, BuilderError> {
            return wher.find(what).ok_or(CouldNotReadLine (id, what));
        }

        let mut read = fs::read_to_string(path).map_err(|e| IOError(e))?;
        let mut res = Vec::new();
        read.retain(|c| c != ' ');

        for (id, line) in read.lines().enumerate() {
            if line.is_empty() { continue; }
            let c_beg = find_or_err(id, &line, "(")?;
            let c_end = find_or_err(id, &line, ")")?;
            let v_beg = find_or_err(id, &line, "[")?;
            let v_end = find_or_err(id, &line, "]")?;
            let c_spl = find_or_err(id, &line[c_beg..c_end], ",")? + c_beg;
            let temp_id: u32 = line[0..c_beg].parse().map_err(|_|CouldNotReadLine (id, "id"))?;
            let coord_x: f32 = line[c_beg+1..c_spl].parse().map_err(|_|CouldNotReadLine (id, "x"))?;
            let coord_y: f32 = line[c_spl+1..c_end].parse().map_err(|_|CouldNotReadLine (id, "y"))?;
            let mut temp_vect = Vec::new();
            for to in line[v_beg+1..v_end].split(',') {
                if !to.is_empty() {
                    temp_vect.push(to.parse::<u32>().map_err(|_| CouldNotReadLine (id, "child"))?);
                }
            }
            res.push((temp_id, (coord_x, coord_y), temp_vect));
        }
        return Ok(res);
    }

    fn template_to_waypoints(t: Vec<(u32, (f32, f32), Vec<u32>)>) -> Result<Vec<Rc<Waypoint>>, BuilderError> {
        use BuilderError::*;
        let mut res = HashMap::new();
        for way in &t {
            if res.contains_key(&way.0) { return Err(RepeatedPoint(way.0)); }
            res.insert(way.0, Rc::new(Waypoint::new(way.1)));
        }
        for way in &t {
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
