use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;

pub fn search(filename: &str, num_projects: usize, num_tries: usize, num_mutations: usize) -> io::Result<String> {
    let prefs = PrefGrid::from_file(filename)?;
    let people = prefs.all_people();
    let mut best: Option<Candidate> = Option::None;
    for _ in 0..num_tries {
        let mut c = Candidate::new(&people, num_projects, &mut thread_rng());
        for _ in 0..num_mutations {
            let mut copy = c.clone();
            copy.mutate(&mut thread_rng());
            if c.score(&prefs) < copy.score(&prefs) {
                c = copy;
            }
        }
        best = Some(if let Some(b) = best {
            if c.score(&prefs) < b.score(&prefs) {b} else {c}
        } else {c});
    }
    Ok(best.unwrap().report(&prefs))
}

#[derive(Debug,Clone)]
pub struct PrefGrid {
    prefs: BTreeMap<String,BTreeSet<String>>
}

impl PrefGrid {
    pub fn from_file(filename: &str) -> io::Result<PrefGrid> {
        let mut rdr = csv::Reader::from_reader(File::open(filename)?);
        let mut result = PrefGrid {prefs: BTreeMap::new()};
        let mut headers = Vec::new();
        for header in rdr.headers()?.iter() {
            if header.len() > 0 {
                result.prefs.insert(String::from(header), BTreeSet::new());
                headers.push(String::from(header));
            }
        }
        for row in rdr.records() {
            let row = row?;
            let contents: Vec<&str> = row.iter().collect();
            for i in 1..contents.len() {
                if contents[i].len() > 0 {
                    result.prefs.get_mut(headers[i - 1].as_str()).unwrap().insert(String::from(contents[0]));
                }
            }
        }
        println!("{:?}", result);
        Ok(result)
    }

    pub fn likes(&self, member: &str, team: &str) -> bool {
        self.prefs.get(member).unwrap().contains(team)
    }

    pub fn all_people(&self) -> Vec<String> {
        self.prefs.iter().map(|(k,_)| String::from(k)).collect()
    }
}

#[derive(Debug,Clone)]
pub struct Candidate {
    projects: Vec<String>,
    members: Vec<Vec<String>>
}

impl Candidate {
    pub fn new<R:Rng>(people: &Vec<String>, num_projects: usize, rng: &mut R) -> Candidate {
        let mut projects = people.clone();
        projects.shuffle(rng);
        while projects.len() > num_projects {
            projects.pop();
        }
        projects.sort();

        let mut candidates = people.clone();
        candidates.shuffle(rng);
        let extra = candidates.len() % num_projects;
        let base_team_size = candidates.len() / num_projects;

        let mut members = Vec::new();
        for _ in 0..num_projects {members.push(Vec::new());}
        let mut team = 0;
        let mut team_size = base_team_size;
        while candidates.len() > 0 {
            if members[team].len() < team_size {
                members[team].push(candidates.pop().unwrap());
            } else {
                team += 1;
                team_size = base_team_size;
                if extra > 0 {
                    team_size += 1;
                }
            }
        }

        Candidate {projects, members}
    }

    pub fn mutate<R:Rng>(&mut self, rng: &mut R) {
        let one = rng.gen_range(0, self.members.len());
        let two = rng.gen_range(0, self.members.len());
        if one != two {
            let a = self.members[one].pop().unwrap();
            let b = self.members[two].pop().unwrap();
            self.members[one].push(b);
            self.members[two].push(a);
        }
    }

    pub fn report(&self, prefs: &PrefGrid) -> String {
        let mut result = String::new();
        for i in 0..self.members.len() {
            result.push_str(self.projects[i].as_str());
            result.push(':');
            for member in self.members[i].iter() {
                result.push(' ');
                result.push_str(member);
            }
            result.push('\n');
        }
        result.push_str(format!("Score: {}", self.score(&prefs)).as_str());
        result
    }

    pub fn score(&self, prefs: &PrefGrid) -> usize {
        let mut result = 0;
        for i in 0..self.members.len() {
            let project = self.projects[i].as_str();
            for member in self.members[i].iter() {
                if prefs.likes(member.as_str(), project) {
                    result += 1;
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::searcher::{PrefGrid, Candidate};
    use std::io;
    use rand::rngs::mock::StepRng;

    #[test]
    pub fn test1() -> io::Result<()> {
        let prefs =PrefGrid::from_file("test1.csv")?;
        assert_eq!(format!("{:?}", prefs), r#"PrefGrid { prefs: {"A": {"B", "D"}, "B": {"A", "C", "D"}, "C": {"A", "B"}, "D": {"B"}} }"#);
        assert!(!prefs.likes("A", "A"));
        assert!(prefs.likes("A", "B"));
        assert!(!prefs.likes("A", "C"));
        assert!(prefs.likes("A", "D"));
        assert!(prefs.likes("B", "A"));
        assert!(!prefs.likes("B", "B"));
        assert!(prefs.likes("B", "C"));
        assert!(prefs.likes("B", "D"));
        assert!(prefs.likes("C", "A"));
        assert!(prefs.likes("C", "B"));
        assert!(!prefs.likes("C", "C"));
        assert!(!prefs.likes("C", "D"));
        assert!(!prefs.likes("D", "A"));
        assert!(prefs.likes("D", "B"));
        assert!(!prefs.likes("D", "C"));
        assert!(!prefs.likes("D", "D"));

        let mut my_rng = StepRng::new(2, 1);
        let candidate = Candidate::new(&prefs.all_people(), 4, &mut my_rng);
        assert_eq!(format!("{:?}", candidate).as_str(), r#"Candidate { projects: ["A", "B", "C", "D"], members: [["A"], ["D"], ["C"], ["B"]] }"#);
        assert_eq!(candidate.score(&prefs), 2);
        Ok(())
    }
}