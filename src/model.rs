use std::{fmt, num::ParseFloatError, str::FromStr};

use geoutils::Location;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Airport {
    pub ident: String,
    pub kind: String,
    pub name: String,
    pub elevation_ft: Option<i32>,
    pub continent: String,
    pub iso_country: String,
    pub iso_region: String,
    pub municipality: String,
    pub gps_code: String,
    pub iata_code: String,
    pub local_code: String,
    pub coordinates: Coords,
}

impl Airport {
    pub fn from_template(template: AirportTemplate) -> Option<Self> {
        let AirportTemplate {
            ident,
            kind,
            name,
            elevation_ft,
            continent,
            iso_country,
            iso_region,
            municipality,
            gps_code,
            iata_code,
            local_code,
            latitude_deg,
            longitude_deg,
        } = template;

        Some(Airport {
            ident,
            kind,
            name,
            elevation_ft,
            continent,
            iso_country,
            iso_region,
            municipality,
            gps_code,
            iata_code,
            local_code,
            coordinates: Coords {
                latitude: latitude_deg,
                longitude: longitude_deg,
            },
        })
    }
}

impl fmt::Display for Airport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.elevation_ft {
            Some(elevation) => write!(
                f,
                "{} {} ({} feet)\n  {}\n  {}\n  {}\n  {}",
                self.ident,
                self.name,
                elevation,
                self.kind,
                self.municipality,
                self.iso_region,
                self.coordinates
            ),

            None => write!(
                f,
                "{} {}\n  {}\n  {}\n  {}\n  {}",
                self.ident,
                self.name,
                self.kind,
                self.municipality,
                self.iso_region,
                self.coordinates
            ),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AirportTemplate {
    ident: String,
    #[serde(rename = "type")]
    kind: String,
    name: String,
    elevation_ft: Option<i32>,
    continent: String,
    iso_country: String,
    iso_region: String,
    municipality: String,
    gps_code: String,
    iata_code: String,
    local_code: String,
    latitude_deg: f64,
    longitude_deg: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Coords {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coords {
    pub fn location(&self) -> Location {
        let &Coords {
            latitude,
            longitude,
        } = self;
        Location::new(latitude, longitude)
    }
}

impl fmt::Display for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = if self.latitude >= 0.0 { "N" } else { "S" };
        let e = if self.longitude >= 0.0 { "E" } else { "W" };

        let lat = self.latitude.abs();
        let lon = self.longitude.abs();

        write!(f, "{lat:.04}°{n} {lon:.04}°{e}")
    }
}

impl FromStr for Coords {
    type Err = ParseCoordsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut values = s.split_ascii_whitespace();
        let latitude: f64 = values
            .next()
            .ok_or(ParseCoordsError::MissingComponent)?
            .parse()?;
        let longitude: f64 = values
            .next()
            .ok_or(ParseCoordsError::MissingComponent)?
            .parse()?;

        if values.next().is_some() {
            return Err(ParseCoordsError::TooManyComponents);
        }

        Ok(Coords {
            latitude,
            longitude,
        })
    }
}

pub enum ParseCoordsError {
    MissingComponent,
    TooManyComponents,
    Float(ParseFloatError),
}

impl From<ParseFloatError> for ParseCoordsError {
    fn from(value: ParseFloatError) -> Self {
        ParseCoordsError::Float(value)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn can_parse_coordinates() {
        todo!()
    }
}
