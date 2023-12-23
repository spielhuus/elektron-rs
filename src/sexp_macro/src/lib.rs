extern crate proc_macro2;
use proc_macro2::{TokenStream, Delimiter, Ident, TokenTree};
use quote::quote;

#[derive(Debug)]
enum SexpState {
    None,
    StartElement,
    Value,
    Text,
    EndElement,
}

struct Parser {
    state: SexpState,
    sexp_builder: Option<Ident>,
}

impl Parser {
    fn parse(&mut self, input: TokenStream, out: &mut Vec<TokenStream>) {
        for token in input {
            match token {
                TokenTree::Group(g) => {

                    if g.delimiter() == Delimiter::Parenthesis {
                        self.state = SexpState::StartElement;
                        self.parse(g.stream(), out);
                        let builder = self.sexp_builder.clone().unwrap();
                        out.push(quote!{
                            #builder.end();
                        });
                        self.state = SexpState::EndElement;
                    } else if g.delimiter() == Delimiter::Brace {
                        match self.state {
                            SexpState::None => todo!(),
                            SexpState::StartElement => todo!(),
                            SexpState::Value => {
                                let builder = self.sexp_builder.clone().unwrap();
                                let value = g.stream();
                                out.push(quote!{
                                     #builder.value(#value);
                                });
                            },
                            SexpState::Text => {
                                let builder = self.sexp_builder.clone().unwrap();
                                let value = g.stream();
                                out.push(quote!{
                                     #builder.text(#value);
                                });
                                self.state = SexpState::Value;
                            },
                            SexpState::EndElement => todo!(),
                        }

                    }
                }
                TokenTree::Ident(i) => {
                    match self.state {
                        SexpState::None => {
                            self.sexp_builder = Some(i);
                        },
                        SexpState::StartElement => {
                            let builder = self.sexp_builder.clone().unwrap();
                            out.push(quote!{
                                #builder.push(stringify!(#i));
                            });
                            self.state = SexpState::Value;
                        },
                        SexpState::Value => {
                            let builder = self.sexp_builder.clone().unwrap();
                            out.push(quote!{
                                #builder.value(#i);
                            });
                        },
                        SexpState::Text => todo!("ident Text"),
                        SexpState::EndElement => todo!("ident EndElement"),
                    }
                }
                TokenTree::Punct(p) => {
                    let punt = p.to_string();
                    if punt == "!" {
                        self.state = SexpState::Text;
                    } // TODO ERROR
                }
                TokenTree::Literal(t) => {
                    match self.state {
                        SexpState::None => todo!("literal None"),
                        SexpState::StartElement => {
                            let builder = self.sexp_builder.clone().unwrap();
                            out.push(quote!{
                                #builder.push(#t);
                            });
                                // #self.sexp_builder.start_element(#t);
                            self.state = SexpState::Value;
                        },
                        SexpState::Value => {
                            let builder = self.sexp_builder.clone().unwrap();
                            let value = t.to_string();
                            if value.starts_with('r') {
                                let value = value.split('"').nth(1).unwrap_or("");
                                out.push(quote!{
                                    #builder.text(#value);
                                });
                            } else {
                                out.push(quote!{
                                    #builder.value(#t);
                                });
                            }
                        },
                        SexpState::Text => {
                            let builder = self.sexp_builder.clone().unwrap();
                            out.push(quote!{
                                #builder.text(#t);
                            });
                        },
                        SexpState::EndElement => {
                            let builder = self.sexp_builder.clone().unwrap();
                            out.push(quote!{
                                #builder.value(#t);
                            });
                        },
                    }
                }
            }
        }
    }
}

#[proc_macro]
pub fn parse_sexp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut tokens: Vec<TokenStream> = vec![];
    let mut parser = Parser{ state: SexpState::None, sexp_builder: None, };
    parser.parse(input.into(), &mut tokens);
    
    let res = quote! {
        #(#tokens)*
    };

    res.into()
}

