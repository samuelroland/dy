use std::{
    ffi::{OsStr, OsString},
    path::PathBuf,
};

pub mod course;
pub mod exo;
pub mod skill;

pub use course::parse_course;
pub use exo::parse_exos;
pub use skill::parse_skills;

// The PLX spec define that course file can only be described inside a `course.dy` and that skills only inside a `skills.dy`
const COURSE_FILE: &str = "course.dy";
const SKILLS_FILE: &str = "skills.dy";
