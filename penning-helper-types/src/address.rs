pub struct Address {
    pub name: String,
    pub street_and_number: String,
    pub postal_code: String,
    pub city: String,
    pub country: Option<String>,
}

impl Address {
    pub fn new(
        name: impl ToString,
        street_and_number: impl ToString,
        postal_code: impl ToString,
        city: impl ToString,
        country: Option<impl ToString>,
    ) -> Self {
        Self {
            name: name.to_string(),
            street_and_number: street_and_number.to_string(),
            postal_code: postal_code.to_string(),
            city: city.to_string(),
            country: country.map(|c| c.to_string()),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        AddressIter {
            address: self,
            index: 0,
        }
    }

    pub fn iter_with_empty(&self) -> impl Iterator<Item = &str> {
        AddressIter {
            address: self,
            index: -1,
        }
    }
}

struct AddressIter<'a> {
    address: &'a Address,
    index: i8,
}

impl<'a> Iterator for AddressIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        match self.index {
            0 => Some(""),
            1 => Some(&self.address.name),
            2 => Some(&self.address.street_and_number),
            3 => Some(&self.address.postal_code),
            4 => Some(&self.address.city),
            5 => self.address.country.as_deref(),
            _ => None,
        }
    }
}
