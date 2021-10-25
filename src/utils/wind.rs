#[derive(Debug)]
pub struct Wind {
    is_variable_direction: bool,
    is_gusting: bool,
    direction: Option<u32>,
    speed: Option<u32>,
    gust_speed: Option<u32>,
}

pub struct WindBuilder<'a> {
    variable: bool,
    direction: &'a str,
    speed: &'a str,
    gust_speed: Option<&'a str>,
}

impl<'a> WindBuilder<'a> {

    pub fn new() -> WindBuilder<'a> {
        WindBuilder {
            variable: false,
            direction: Default::default(),
            speed: Default::default(),
            gust_speed: Default::default(),
        }
    }

    pub fn is_variable(&mut self) -> &mut Self {
        self.variable = true;
        self
    }

    pub fn with_direction(&mut self, direction: &'a str) -> &mut Self {
        self.direction = direction;
        self
    }

    pub fn with_speed(&mut self, speed: &'a str) -> &mut Self {
        self.speed = speed;
        self
    }

    pub fn with_gust_speed(&mut self, gust_speed: Option<&'a str>) -> &mut Self {
        self.gust_speed = gust_speed;
        self
    }

    pub fn build(&self) -> Wind {
        Wind {
            is_variable_direction: self.variable,
            direction: match self.direction.parse::<u32>() {
                Ok(direction) => Some(direction),
                Err(error) => None
            },
        
            speed: match self.speed.parse::<u32>() {
                Ok(speed) => Some(speed),
                Err(error) => None
            },
        
            gust_speed: match self.gust_speed {
                Some(gust_speed) => Some(self.gust_speed.unwrap().parse::<u32>().unwrap()),
                None => None
            },

            is_gusting: match self.gust_speed {
                Some(gust_speed) => true,
                None => false
            },
        }
    }

}

