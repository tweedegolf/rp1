use std::str::FromStr;

use crate::ParseError;

/// Supported filter operators for the filter parameter.
///
/// A filter can be added to the list request by using a query parameter.
/// These query parameters should follow this general pattern:
/// `?filter[field].ne=foo&filter[other]=bar`. The filter syntax allows for
/// some variations of the `filter[field].op=value` syntax:
///
/// * `filter[field].op=value`
/// * `filter.field.op=value`
/// * `filter[field]op=value`
///
#[derive(Debug)]
pub enum FilterOperator<T> {
    /// Equals filter, i.e. `filter[field].eq=value`.
    ///
    /// Only records that have the given value for the given field will be
    /// returned. This is also the default filter type if none is given, so
    /// `filter[field]=value` has the same meaning.
    Eq(T),

    /// Not equals filter, i.e. `filter[field].ne=value`.
    ///
    /// Only records that have a different value than the given value in the
    /// given field will be returned.
    Ne(T),

    /// Greater than filter, i.e. `filter[field].gt=value`.
    ///
    /// Only records that have a value that is greater than the given value in
    /// the given field will be returned.
    Gt(T),

    /// Greater than or equals filter, i.e. `filter[field].ge=value`
    ///
    /// Only records that have a value that is greater than or that equal the
    /// given value in the given field will be returned.
    Ge(T),

    /// Less than filter, i.e. `filter[field].lt=value`.
    ///
    /// Only records that have a value that is less than the given value in
    /// the given field will be returned.
    Lt(T),

    /// Less than or equals filter, i.e. `filter[field].le=value`
    ///
    /// Only records that have a value that is less than or equal to the
    /// given value in the given field will be returned.
    Le(T),

    /// Equals any filter, i.e. `filter[field].in=value1,value2,value3`
    ///
    /// Only records that have one of the given values in the given field will
    /// be returned. Note that the values themselves may not contain a comma,
    /// as then they will be interpreted as separate values. It is not
    /// recommended to use this filter for unstructured user data.
    EqAny(Vec<T>),
}

/// Parser function for filters that expect a single filter value
fn parse_single_operand<T: FromStr>(input: &str) -> Result<T, ParseError>
where
    <T as FromStr>::Err: Into<ParseError>,
{
    input.parse().map_err(|e: <T as FromStr>::Err| e.into())
}

/// Parser function for filters that expect a list of filter values
///
/// Note that the vec filter operands currently split based on comma, if you
/// have a comma in your value there is currently no escaping method.
fn parse_vec_operand<T: FromStr>(input: &str) -> Result<Vec<T>, ParseError>
where
    <T as FromStr>::Err: Into<ParseError>,
{
    input
        .split(',')
        .map(|segment| segment.parse())
        .collect::<Result<Vec<T>, <T as FromStr>::Err>>()
        .map_err(|e: <T as FromStr>::Err| e.into())
}

impl<T: FromStr> FilterOperator<Option<T>>
where
    <T as FromStr>::Err: Into<ParseError>,
{
    pub fn from_none(op: &str) -> Result<Self, ParseError> {
        match op {
            "eq" => Ok(FilterOperator::Eq(None)),
            "ne" => Ok(FilterOperator::Ne(None)),
            "gt" => Ok(FilterOperator::Gt(None)),
            "ge" => Ok(FilterOperator::Ge(None)),
            "lt" => Ok(FilterOperator::Lt(None)),
            "le" => Ok(FilterOperator::Le(None)),
            "in" => Ok(FilterOperator::EqAny(vec![])),
            _ => Err(ParseError::UnknownOperator(op.to_owned())),
        }
    }

    pub fn try_parse_option(op: &str, value: &str) -> Result<Self, ParseError> {
        let value: FilterOperator<T> = FilterOperator::try_parse(op, value)?;
        Ok(match value {
            FilterOperator::Eq(v) => FilterOperator::Eq(Some(v)),
            FilterOperator::Ne(v) => FilterOperator::Ne(Some(v)),
            FilterOperator::Gt(v) => FilterOperator::Ne(Some(v)),
            FilterOperator::Ge(v) => FilterOperator::Ne(Some(v)),
            FilterOperator::Lt(v) => FilterOperator::Ne(Some(v)),
            FilterOperator::Le(v) => FilterOperator::Ne(Some(v)),
            FilterOperator::EqAny(v) => FilterOperator::EqAny(v.into_iter().map(Some).collect()),
        })
    }
}

impl<T: FromStr> FilterOperator<T>
where
    <T as FromStr>::Err: Into<ParseError>,
{
    pub fn try_parse(op: &str, value: &str) -> Result<Self, ParseError> {
        match op {
            "eq" => Ok(FilterOperator::Eq(parse_single_operand(value)?)),
            "ne" => Ok(FilterOperator::Ne(parse_single_operand(value)?)),
            "gt" => Ok(FilterOperator::Gt(parse_single_operand(value)?)),
            "ge" => Ok(FilterOperator::Ge(parse_single_operand(value)?)),
            "lt" => Ok(FilterOperator::Lt(parse_single_operand(value)?)),
            "le" => Ok(FilterOperator::Le(parse_single_operand(value)?)),
            "in" => Ok(FilterOperator::EqAny(parse_vec_operand(value)?)),
            _ => Err(ParseError::UnknownOperator(op.to_owned())),
        }
    }
}
