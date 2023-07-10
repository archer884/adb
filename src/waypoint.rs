use std::fmt;

use geoutils::Distance;

use crate::model::{Airport, Coords};

// Practically all instances of Waypoint will be the Airport variant.
#[allow(clippy::large_enum_variant)]
pub enum Waypoint {
    Airport(Airport),
    Coords(Coords),
}

impl From<Airport> for Waypoint {
    fn from(value: Airport) -> Self {
        Waypoint::Airport(value)
    }
}

impl From<Coords> for Waypoint {
    fn from(value: Coords) -> Self {
        Waypoint::Coords(value)
    }
}

impl Waypoint {
    pub fn name(&self) -> WaypointName {
        WaypointName { waypoint: self }
    }

    pub fn distance_to(&self, other: &Waypoint) -> Distance {
        let left = self.coordinates().location();
        let right = other.coordinates().location();

        // I have never, ever, ever seen Vicenty's formula fail to yield a result, but IF IT DOES
        // we'll fall back to haversine distance.
        left.distance_to(&right)
            .unwrap_or_else(|_| left.haversine_distance_to(&right))
    }

    fn coordinates(&self) -> Coords {
        match self {
            Waypoint::Airport(airport) => airport.coordinates,
            Waypoint::Coords(coordinates) => *coordinates,
        }
    }
}

pub struct WaypointName<'a> {
    waypoint: &'a Waypoint,
}

impl fmt::Display for WaypointName<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.waypoint {
            Waypoint::Airport(airport) => airport.ident.fmt(f),
            Waypoint::Coords(coords) => coords.fmt(f),
        }
    }
}
