pub mod course;
pub mod exo;
pub mod skill;

pub use dy;

pub use course::parse_course;
pub use exo::parse_exo;
pub use skill::parse_skills;

// The PLX spec define that course file can only be described inside a `course.dy` and that skills only inside a `skills.dy`
pub const COURSE_FILE: &str = "course.dy";
pub const SKILLS_FILE: &str = "skills.dy";
pub const EXO_FILE: &str = "exo.dy";
