use Renderable;
use Block;
use value::Value;
use context::Context;
use template::Template;
use LiquidOptions;
use tags::IfBlock;
use lexer::Token;
use lexer::Token::{Identifier, StringLiteral, NumberLiteral, Comparison};
use lexer::ComparisonOperator;
use lexer::ComparisonOperator::{
    Equals, NotEquals,
    LessThan, GreaterThan,
    LessThanEquals, GreaterThanEquals,
    Contains
};
use parser::parse;
use lexer::Element;
use lexer::Element::Tag;

#[cfg(test)]
use std::default::Default;
#[cfg(test)]
use lexer::Element::Raw;

struct If<'a>{
    lh : Token,
    comparison : ComparisonOperator,
    rh : Token,
    if_true: Template<'a>,
    if_false: Option<Template<'a>>
}

impl<'a> If<'a>{
    fn compare(&self, context: &Context) -> Result<bool, &'static str>{
        match (&self.lh, &self.rh)  {
            (&NumberLiteral(a), &NumberLiteral(b)) => Ok(compare_numbers(a, b, &self.comparison)),
            (&Identifier(ref var), &NumberLiteral(b)) => {
                match context.get_val(var) {
                    Some(&Value::Num(a)) => Ok(compare_numbers(a, b, &self.comparison)),
                    _ => Err("not comparable")
                }
            },
            (&NumberLiteral(a), &Identifier(ref var)) => {
                match context.get_val(var) {
                    Some(&Value::Num(b)) => Ok(compare_numbers(a, b, &self.comparison)),
                    _ => Err("not comparable")
                }
            }
            (&Identifier(ref var_a), &Identifier(ref var_b)) => {
                match (context.get_val(var_a), context.get_val(var_b)) {
                    (Some(&Value::Num(a)), Some(&Value::Num(b))) => Ok(compare_numbers(a, b, &self.comparison)),
                    _ => Err("not comparable")
                }
            }
            (_, _) => Err("not implemented yet!") // TODO
        }
    }
}

// TODO surely there's a nicer way for this
fn compare_numbers(a : f32, b : f32, comparison : &ComparisonOperator) -> bool{
    match comparison {
        &Equals => a == b,
        &NotEquals => a != b,
        &LessThan => a < b,
        &GreaterThan => a > b,
        &LessThanEquals => a <= b,
        &GreaterThanEquals => a >= b,
        &Contains => false, // TODO!!!
    }
}

impl<'a> Renderable for If<'a>{
    fn render(&self, context: &mut Context) -> Option<String>{
        if self.compare(context).unwrap_or(false){
            self.if_true.render(context)
        }else{
            match self.if_false {
                Some(ref template) => template.render(context),
                _ => None
            }
        }
    }
}

impl Block for IfBlock{
    fn initialize<'a>(&'a self, _tag_name: &str, arguments: &[Token], tokens: Vec<Element>, options : &'a LiquidOptions) -> Result<Box<Renderable +'a>, String>{
        let mut args = arguments.iter();

        let lh = match args.next() {
            Some(&StringLiteral(ref x)) => StringLiteral(x.clone()),
            Some(&NumberLiteral(ref x)) => NumberLiteral(x.clone()),
            Some(&Identifier(ref x)) => Identifier(x.clone()),
            x => return Err(format!("Expected a value, found {:?}", x))
        };

        let comp = match args.next() {
            Some(&Comparison(ref x)) => x.clone(),
            x => return Err(format!("Expected a comparison operator, found {:?}", x))
        };

        let rh = match args.next() {
            Some(&StringLiteral(ref x)) => StringLiteral(x.clone()),
            Some(&NumberLiteral(ref x)) => NumberLiteral(x.clone()),
            Some(&Identifier(ref x)) => Identifier(x.clone()),
            x => return Err(format!("Expected a value, found {:?}", x))
        };

        let else_block = vec![Identifier("else".to_string())];

        // advance until the end or an else token is reached
        // to gather everything to be executed if the condition is true
        let if_true_tokens : Vec<Element> = tokens.iter().take_while(|&x| match x  {
            &Tag(ref eb, _) => *eb != else_block,
            _ => true
        }).map(|x| x.clone()).collect();

        // gather everything after the else block
        // to be executed if the condition is false
        let if_false_tokens : Vec<Element> = tokens.iter().skip_while(|&x| match x  {
            &Tag(ref eb, _) => *eb != else_block,
            _ => true
        }).skip(1).map(|x| x.clone()).collect();

        // if false is None if there is no block to execute
        let if_false = if if_false_tokens.len() > 0 {
            Some(Template::new(try!(parse(&if_false_tokens, options))))
        }else{
            None
        };

        let if_true = Template::new(try!(parse(&if_true_tokens, options)));

        Ok(box If{
            lh : lh,
            comparison : comp,
            rh : rh,
            if_true: if_true,
            if_false: if_false
        } as Box<Renderable>)
    }
}

#[test]
fn test_if() {
    let block = IfBlock;
    let options : LiquidOptions = Default::default();
    // 5 < 6 then "if true" else "if false"
    let if_tag = block.initialize("if", &vec![NumberLiteral(5f32), Comparison(LessThan), NumberLiteral(6f32)], vec![Raw("if true".to_string())], &options);
    assert_eq!(if_tag.unwrap().render(&mut Default::default()).unwrap(), "if true".to_string());

    // 7 < 6 then "if true" else "if false"
    let else_tag = block.initialize("if", &vec![NumberLiteral(7f32), Comparison(LessThan), NumberLiteral(6f32)], vec![Raw("if true".to_string()), Tag(vec![Identifier("else".to_string())], "".to_string()), Raw("if false".to_string())], &options);
    assert_eq!(else_tag.unwrap().render(&mut Default::default()).unwrap(), "if false".to_string());
}

