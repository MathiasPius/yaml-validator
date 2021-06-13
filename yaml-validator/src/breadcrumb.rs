#![macro_use]

#[cfg(feature = "smallvec")]
pub type BreadcrumbSegmentVec<'a> = smallvec::SmallVec<[BreadcrumbSegment<'a>; 8]>;
#[cfg(not(feature = "smallvec"))]
pub type BreadcrumbSegmentVec<'a> = Vec<BreadcrumbSegment<'a>>;

#[cfg(test)]
#[cfg(feature = "smallvec")]
macro_rules! breadcrumb{
    ( $( $x:expr ),* ) => {
        smallvec::smallvec![
            $(crate::breadcrumb::BreadcrumbSegment::from($x),)*
        ]
    }
}

#[cfg(test)]
#[cfg(not(feature = "smallvec"))]
macro_rules! breadcrumb{
    ( $( $x:expr ),* ) => {
        vec![
            $(crate::breadcrumb::BreadcrumbSegment::from($x),)*
        ]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BreadcrumbSegment<'a> {
    Name(&'a str),
    Index(usize),
}

impl<'a> From<&'a str> for BreadcrumbSegment<'a> {
    fn from(name: &'a str) -> Self {
        BreadcrumbSegment::Name(name)
    }
}

impl<'a> From<usize> for BreadcrumbSegment<'a> {
    fn from(index: usize) -> Self {
        BreadcrumbSegment::Index(index)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Breadcrumb<'a> {
    segments: BreadcrumbSegmentVec<'a>,
}

impl<'a> Breadcrumb<'a> {
    pub fn new(segments: BreadcrumbSegmentVec<'a>) -> Self {
        Breadcrumb { segments }
    }
    pub fn push(&mut self, segment: BreadcrumbSegment<'a>) {
        self.segments.push(segment);
    }
}

impl<'a> std::fmt::Display for Breadcrumb<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in self.segments.iter().rev() {
            match segment {
                BreadcrumbSegment::Name(name) => write!(f, ".{}", name)?,
                BreadcrumbSegment::Index(index) => write!(f, "[{}]", index)?,
            };
        }

        Ok(())
    }
}

impl<'a> Default for Breadcrumb<'a> {
    fn default() -> Self {
        Breadcrumb {
            segments: BreadcrumbSegmentVec::new(),
        }
    }
}
