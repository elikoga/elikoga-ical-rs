use std::ops::Range;

use elikoga_ical_rs::{ContentLine, ICalObject, Param};
use rand::{distributions::Alphanumeric, prelude::Distribution};

struct ObjectDistribution {
    type_distribution: NameDistribution,
    property_count_range: Range<usize>,
    property_distribution: PropertyDistribution,
    sub_object_count_range_distribution: Option<(Range<usize>, Box<ObjectDistribution>)>,
}

struct PropertyDistribution {
    name_distribution: NameDistribution,
    value_length_range: ValueDistribution,
    param_count_range: Range<usize>,
    param_distribution: ParamDistribution,
}

impl Default for PropertyDistribution {
    fn default() -> Self {
        PropertyDistribution {
            name_distribution: NameDistribution::default(),
            value_length_range: ValueDistribution::default(),
            param_count_range: (0..10),
            param_distribution: ParamDistribution::default(),
        }
    }
}

impl Default for ObjectDistribution {
    fn default() -> Self {
        ObjectDistribution {
            type_distribution: NameDistribution::default(),
            property_count_range: (0..10),
            property_distribution: PropertyDistribution::default(),
            sub_object_count_range_distribution: None,
        }
    }
}

struct ParamDistribution {
    name_distribution: NameDistribution,
    value_count_range: Range<usize>,
    value_distribution: ValueDistribution,
}

impl Default for ParamDistribution {
    fn default() -> Self {
        Self {
            name_distribution: NameDistribution::default(),
            value_count_range: (0..50),
            value_distribution: ValueDistribution::default(),
        }
    }
}

impl Distribution<Param> for ParamDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Param {
        let name = self.name_distribution.sample(rng);
        let values = (0..rng.gen_range(self.value_count_range.clone()))
            .map(|_| self.value_distribution.sample(rng))
            .collect();
        Param::new(name, values)
    }
}

impl Distribution<ContentLine> for PropertyDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ContentLine {
        let name = loop {
            // rejection sample to avoid begin
            let name = self.name_distribution.sample(rng);
            if name.eq_ignore_ascii_case("BEGIN") {
                continue;
            }
            break name;
        };
        let value = self.value_length_range.sample(rng);
        let params: Vec<Param> = (0..rng.gen_range(self.param_count_range.clone()))
            .map(|_| self.param_distribution.sample(rng))
            .collect();
        ContentLine::new(name, params, value)
    }
}

impl Distribution<ICalObject> for ObjectDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> ICalObject {
        let mut properties = Vec::new();
        let mut sub_objects = Vec::new();
        for _ in 0..rng.gen_range(self.property_count_range.clone()) {
            properties.push(self.property_distribution.sample(rng));
        }
        if let Some((sub_object_count_range, sub_object_distribution)) =
            self.sub_object_count_range_distribution.as_ref()
        {
            for _ in 0..rng.gen_range(sub_object_count_range.clone()) {
                sub_objects.push(sub_object_distribution.sample(rng));
            }
        }
        let name = self.type_distribution.sample(rng);
        ICalObject {
            object_type: name,
            properties,
            sub_objects,
        }
    }
}

struct NameDistribution {
    name_length_range: Range<usize>,
}

struct QSafeDistribution {
    value_length_range: Range<usize>,
}

struct ValueDistribution {
    value_length_range: Range<usize>,
}

impl Default for ValueDistribution {
    fn default() -> Self {
        ValueDistribution {
            value_length_range: (1..200),
        }
    }
}

impl Default for NameDistribution {
    fn default() -> Self {
        NameDistribution {
            name_length_range: (1..200),
        }
    }
}

impl Default for QSafeDistribution {
    fn default() -> Self {
        QSafeDistribution {
            value_length_range: (1..200),
        }
    }
}

impl Distribution<String> for NameDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> String {
        const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-";
        let name_length = rng.gen_range(self.name_length_range.clone());
        let mut name = String::with_capacity(name_length);
        for _ in 0..name_length {
            let idx = rng.gen_range(0..ALPHABET.len());
            name.push(ALPHABET.chars().nth(idx).unwrap());
        }
        name
    }
}

impl Distribution<String> for QSafeDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> String {
        const ALPHABET: &str = " !#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";
        let value_length = rng.gen_range(self.value_length_range.clone());
        let mut value = String::with_capacity(value_length);
        for _ in 0..value_length {
            let idx = rng.gen_range(0..ALPHABET.len());
            value.push(ALPHABET.chars().nth(idx).unwrap());
        }
        value
    }
}

impl Distribution<String> for ValueDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> String {
        let value_length = rng.gen_range(self.value_length_range.clone());
        let mut value = String::new();
        for _ in 0..value_length {
            // rejection sample: everything except control characters
            let c = loop {
                let c: char = Alphanumeric.sample(rng) as char;
                if (c.is_ascii_control() && c != '\t') || c == '\r' || c == '\n' {
                    continue;
                }
                break c;
            };
            value.push(c);
        }
        value
    }
}

fn main() {
    let object_distribution = ObjectDistribution {
        sub_object_count_range_distribution: Some((
            (0..1000),
            Box::new(ObjectDistribution {
                sub_object_count_range_distribution: Some((
                    (0..10),
                    Box::new(ObjectDistribution {
                        sub_object_count_range_distribution: None,
                        ..ObjectDistribution::default()
                    }),
                )),
                ..ObjectDistribution::default()
            }),
        )),
        ..ObjectDistribution::default()
    };
    let mut rng = rand::thread_rng();
    let mut object = object_distribution.sample(&mut rng);
    // set object type to VCALENDAR
    object.object_type = "VCALENDAR".to_string();
    print!("{}", object);
}
