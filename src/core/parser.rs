use crate::core::datamodel::{
    CFrameWrapper, Color3Wrapper, PropertyValue, UDim2Wrapper, Vec3Wrapper,
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alpha1, char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::many0,
    sequence::{delimited, pair},
};
use std::collections::HashMap;

// --- Parsers ---

fn ws<'a, F, O, E: nom::error::ParseError<&'a str>>(
    inner: F,
) -> impl Parser<&'a str, Output = O, Error = E>
where
    F: Parser<&'a str, Output = O, Error = E>,
{
    delimited(multispace0, inner, multispace0)
}

fn parse_identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ))
    .parse(input)
}

fn parse_string(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("\"")(input)?;
    let (input, content) = take_while(|c| c != '"')(input)?;
    let (input, _) = tag("\"")(input)?;
    Ok((input, content.to_string()))
}

fn parse_number(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize((
            opt(char('-')),
            digit1,
            opt(pair(char('.'), digit1)),
        )),
        |s: &str| s.parse::<f64>(),
    )
    .parse(input)
}

fn parse_enum(input: &str) -> IResult<&str, PropertyValue> {
    // Enum.PartType.Block
    let (input, _) = tag("Enum.")(input)?;
    let (input, enum_type) = parse_identifier(input)?;
    let (input, _) = char('.')(input)?;
    let (input, enum_item) = parse_identifier(input)?;
    Ok((input, PropertyValue::Enum(format!("Enum.{}.{}", enum_type, enum_item))))
}

fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false))).parse(input)
}

// Constructors

fn parse_vector3(input: &str) -> IResult<&str, PropertyValue> {
    let (input, _) = tag("Vector3.new")(input)?;
    let (input, _) = ws(char('(')).parse(input)?;
    let (input, x) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, y) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, z) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(')')).parse(input)?;
    Ok((
        input,
        PropertyValue::Vector3(Vec3Wrapper {
            x: x as f32,
            y: y as f32,
            z: z as f32,
        }),
    ))
}

fn parse_cframe(input: &str) -> IResult<&str, PropertyValue> {
    let (input, _) = tag("CFrame.new")(input)?;
    let (input, _) = ws(char('(')).parse(input)?;
    let (input, x) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, y) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, z) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(')')).parse(input)?;
    Ok((
        input,
        PropertyValue::CFrame(CFrameWrapper::new(x as f32, y as f32, z as f32)),
    ))
}

fn parse_color3_from_rgb(input: &str) -> IResult<&str, PropertyValue> {
    let (input, _) = tag("Color3.fromRGB")(input)?;
    let (input, _) = ws(char('(')).parse(input)?;
    let (input, r) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, g) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, b) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(')')).parse(input)?;
    Ok((
        input,
        PropertyValue::Color3(Color3Wrapper::from_rgb(r as f32, g as f32, b as f32)),
    ))
}

fn parse_color3_new(input: &str) -> IResult<&str, PropertyValue> {
    let (input, _) = tag("Color3.new")(input)?;
    let (input, _) = ws(char('(')).parse(input)?;
    let (input, r) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, g) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, b) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(')')).parse(input)?;
    Ok((
        input,
        PropertyValue::Color3(Color3Wrapper::new(r as f32, g as f32, b as f32)),
    ))
}

fn parse_udim2(input: &str) -> IResult<&str, PropertyValue> {
    let (input, _) = tag("UDim2.new")(input)?;
    let (input, _) = ws(char('(')).parse(input)?;
    let (input, xs) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, xo) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, ys) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(',')).parse(input)?;
    let (input, yo) = ws(parse_number).parse(input)?;
    let (input, _) = ws(char(')')).parse(input)?;
    Ok((
        input,
        PropertyValue::UDim2(UDim2Wrapper {
            xs: xs as f32,
            xo: xo as i32,
            ys: ys as f32,
            yo: yo as i32,
        }),
    ))
}

fn parse_value(input: &str) -> IResult<&str, PropertyValue> {
    alt((
        map(parse_bool, PropertyValue::Bool),
        parse_vector3,
        parse_cframe,
        parse_color3_from_rgb,
        parse_color3_new,
        parse_udim2,
        parse_enum,
        map(parse_number, PropertyValue::Number),
        map(parse_string, PropertyValue::String),
        map(parse_identifier, |s| PropertyValue::String(s.to_string())),
    ))
    .parse(input)
}

fn parse_assignment(input: &str) -> IResult<&str, (String, PropertyValue)> {
    let (input, key) = ws(parse_identifier).parse(input)?;
    let (input, _) = ws(char('=')).parse(input)?;
    let (input, value) = ws(parse_value).parse(input)?;
    Ok((input, (key.to_string(), value)))
}

pub fn parse_instance_dsl(input: &str) -> IResult<&str, HashMap<String, PropertyValue>> {
    let (input, pairs) = many0(parse_assignment).parse(input)?;
    let mut map = HashMap::new();
    for (k, v) in pairs {
        map.insert(k, v);
    }
    Ok((input, map))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vector3() {
        let input = "Vector3.new(10, 20, 30.5)";
        let (_, val) = parse_vector3(input).unwrap();
        if let PropertyValue::Vector3(v) = val {
            assert_eq!(v.x, 10.0);
            assert_eq!(v.y, 20.0);
            assert_eq!(v.z, 30.5);
        } else {
            panic!("Expected Vector3");
        }
    }

    #[test]
    fn test_parse_simple_assignment() {
        let input = "Name = \"TestPart\"";
        let (_, (key, val)) = parse_assignment(input).unwrap();
        assert_eq!(key, "Name");
        assert_eq!(val, PropertyValue::String("TestPart".to_string()));
    }

    #[test]
    fn test_parse_full_dsl() {
        let input = r#"
            ClassName = Part
            Transparency = 0.5
            Anchored = true
            Size = Vector3.new(4, 1, 2)
            Color = Color3.fromRGB(255, 0, 0)
        "#;
        let (_, props) = parse_instance_dsl(input).unwrap();

        assert_eq!(
            props.get("ClassName"),
            Some(&PropertyValue::String("Part".to_string()))
        );
        assert_eq!(props.get("Transparency"), Some(&PropertyValue::Number(0.5)));
        assert_eq!(props.get("Anchored"), Some(&PropertyValue::Bool(true)));

        if let Some(PropertyValue::Vector3(v)) = props.get("Size") {
            assert_eq!(v.x, 4.0);
        } else {
            panic!("Size wrong type");
        }

        if let Some(PropertyValue::Color3(c)) = props.get("Color") {
            assert_eq!(c.r, 1.0); // 255/255
        } else {
            panic!("Color wrong type");
        }
    }
}
