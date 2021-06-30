//a NSPrefixId, NSUriId, NSMap, NameID
//tp NSPrefixId
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NSPrefixId (usize);
impl NSPrefixId {
    pub fn none() -> Self { Self(0) }
    pub fn new(n:usize) -> Self { Self(n+1) }
    pub fn is_none(&self) -> bool { self.0 == 0 }
    pub fn is_some(&self) -> bool { self.0 > 0 }
    pub fn get(&self) -> Option<usize> {
        match self.0 {
            0 => None,
            x => Some(x-1)
        }
    }
}
impl std::fmt::Display for NSPrefixId {
    fn fmt(&self, fmt:&mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            0 => write!(fmt, "None"),
            x => write!(fmt, "{}", x - 1),
        }
    }
}

//tp NSUriId
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NSUriId (usize);
impl NSUriId {
    pub fn none() -> Self { Self(0) }
    pub fn new(n:usize) -> Self { Self(n+1) }
    pub fn is_none(&self) -> bool { self.0 == 0 }
    pub fn is_some(&self) -> bool { self.0 > 0 }
    pub fn get(&self) -> Option<usize> {
        match self.0 {
            0 => None,
            x => Some(x-1)
        }
    }
}
impl std::fmt::Display for NSUriId {
    fn fmt(&self, fmt:&mut std::fmt::Formatter) -> Result<(),std::fmt::Error> {
        match self.0 {
            0 => write!(fmt, "None"),
            x => write!(fmt, "{}", x - 1),
        }
    }
}

//tp NSNameId
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NSNameId (usize);
impl NSNameId {
    pub fn none() -> Self { Self(0) }
    pub fn new(n:usize) -> Self { Self(n+1) }
    pub fn is_none(&self) -> bool { self.0 == 0 }
    pub fn is_some(&self) -> bool { self.0 > 0 }
    pub fn get(&self) -> Option<usize> {
        match self.0 {
            0 => None,
            x => Some(x-1)
        }
    }
}
impl std::fmt::Display for NSNameId {
    fn fmt(&self, fmt:&mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            0 => write!(fmt, "None"),
            x => write!(fmt, "{}", x - 1),
        }
    }
}

//tp NSMap
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NSMap (NSPrefixId, NSUriId);
impl NSMap {
    #[inline]
    pub fn new(p_id:NSPrefixId, u_id:NSUriId) -> Self { Self(p_id, u_id) }
    #[inline]
    pub fn prefix_id(&self) -> NSPrefixId { self.0 }
    pub fn uri_id(&self) -> NSUriId { self.1 }
}
impl std::fmt::Display for NSMap {
    fn fmt(&self, fmt:&mut std::fmt::Formatter) -> Result<(),std::fmt::Error> {
        write!(fmt, "{} => {}",self.0,self.1)
    }
}

