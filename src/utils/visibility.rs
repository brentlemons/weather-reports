#[derive(Debug)]
pub struct Visibility {
    is_unlimited: bool,
    is_unknown: bool,
    is_meters: bool,
    is_statute_miles: bool,
    distance: Option<u32>,
}

pub struct VisibilityBuilder<'a> {
    unlimited: bool,
    unknown: bool,
    meters: bool,
    statute_miles: bool,
    distance: &'a str,
}

impl<'a> VisibilityBuilder<'a> {

    pub fn new() -> VisibilityBuilder<'a> {
        VisibilityBuilder {
            unlimited: false,
            unknown: false,
            meters: false,
            statute_miles: false,
            distance: Default::default(),
        }
    }

    pub fn is_unlimited(&mut self) -> &mut Self {
        self.unlimited = true;
        self
    }

    pub fn is_unknown(&mut self) -> &mut Self {
        self.unknown = true;
        self
    }

    pub fn is_meters(&mut self) -> &mut Self {
        self.meters = true;
        self
    }

    pub fn is_statute_miles(&mut self) -> &mut Self {
        self.statute_miles = true;
        self
    }

    pub fn with_distance(&mut self, distance: &'a str) -> &mut Self {
        self.distance = distance;
        self
    }

    pub fn build(&self) -> Visibility {
        Visibility {
            is_unlimited: self.unlimited,
            is_unknown: self.unknown,
            is_meters: self.meters,
            is_statute_miles: self.statute_miles,
            distance: match self.distance.parse::<u32>() {
                Ok(distance) => Some(distance),
                Err(error) => None
            },
        }
    }

}

