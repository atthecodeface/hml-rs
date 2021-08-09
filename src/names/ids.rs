//a NSPrefixId, NSUriId, NSMap, NameID
//tp NSPrefixId
/// An ID for a prefix, within a `Namespace`
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NSPrefixId(usize);
impl NSPrefixId {
    //fp none
    /// Get an [NSPrefixId] of `None`
    pub fn none() -> Self {
        Self(0)
    }

    //fp new
    /// Get an [NSPrefixId] from an index
    pub fn new(n: usize) -> Self {
        Self(n + 1)
    }

    //mp is_none
    /// Return true if the id is `None`
    pub fn is_none(&self) -> bool {
        self.0 == 0
    }

    //mp is_some
    /// Return true if the id is not `None`
    pub fn is_some(&self) -> bool {
        self.0 > 0
    }

    //mp get
    /// Get the [NSPrefixId] is either None or Some(index)
    pub fn get(&self) -> Option<usize> {
        match self.0 {
            0 => None,
            x => Some(x - 1),
        }
    }
}

//ip Display for NSPrefixId
impl std::fmt::Display for NSPrefixId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            0 => write!(fmt, "None"),
            x => write!(fmt, "{}", x - 1),
        }
    }
}

//tp NSUriId
/// An ID for a URI, within a `Namespace`
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NSUriId(usize);
impl NSUriId {
    //fp none
    /// Get an [NSUriId] of `None`
    pub fn none() -> Self {
        Self(0)
    }

    //fp new
    /// Get an [NSUriId] from an index
    pub fn new(n: usize) -> Self {
        Self(n + 1)
    }

    //mp is_none
    /// Return true if the id is `None`
    pub fn is_none(&self) -> bool {
        self.0 == 0
    }

    //mp is_some
    /// Return true if the id is not `None`
    pub fn is_some(&self) -> bool {
        self.0 > 0
    }

    //mp get
    /// Get the [NSUriId] is either None or Some(index)
    pub fn get(&self) -> Option<usize> {
        match self.0 {
            0 => None,
            x => Some(x - 1),
        }
    }
}

//ip Display for NSUriId
impl std::fmt::Display for NSUriId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self.0 {
            0 => write!(fmt, "None"),
            x => write!(fmt, "{}", x - 1),
        }
    }
}

//tp NSNameId
/// An ID for a Name, within a `Namespace`
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NSNameId(usize);
impl NSNameId {
    //fp none
    /// Get an [NSNameId] of `None`
    pub fn none() -> Self {
        Self(0)
    }

    //fp new
    /// Get an [NSNameId] from an index
    pub fn new(n: usize) -> Self {
        Self(n + 1)
    }

    //mp is_none
    /// Return true if the id is `None`
    pub fn is_none(&self) -> bool {
        self.0 == 0
    }

    //mp is_some
    /// Return true if the id is not `None`
    pub fn is_some(&self) -> bool {
        self.0 > 0
    }

    //mp get
    /// Get the [NSNameId] is either None or Some(index)
    pub fn get(&self) -> Option<usize> {
        match self.0 {
            0 => None,
            x => Some(x - 1),
        }
    }
}

//ip Display for NSNameId
impl std::fmt::Display for NSNameId {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            0 => write!(fmt, "None"),
            x => write!(fmt, "{}", x - 1),
        }
    }
}

//tp NSMap
/// An map between a [NSPrefixId] and an [NSUriId], which must be within the same `Nmespace`
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NSMap(NSPrefixId, NSUriId);
impl NSMap {
    //fp new
    /// Create a new map between an NSPrefixId and an NSUriId
    #[inline]
    pub fn new(p_id: NSPrefixId, u_id: NSUriId) -> Self {
        Self(p_id, u_id)
    }

    //mp prefix_id
    /// Retrieve the NSPrefixId of the mapping
    #[inline]
    pub fn prefix_id(&self) -> NSPrefixId {
        self.0
    }

    //mp uri_id
    /// Retrieve the NSUriId of the mapping
    pub fn uri_id(&self) -> NSUriId {
        self.1
    }

    //zz All done
}

//ip Display for NSMap
impl std::fmt::Display for NSMap {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(fmt, "{} => {}", self.0, self.1)
    }
}
