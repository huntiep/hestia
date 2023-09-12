use types::*;

use std::fmt::Display;

#[derive(BartDisplay)]
#[template = "templates/head.html"]
pub struct TemplateHead<'a> {
    pub title: Option<&'a str>,
}

impl<'a> TemplateHead<'a> {
    pub fn new(title: Option<&'a str>) -> Self {
        TemplateHead {
            title: title,
        }
    }
}

#[derive(BartDisplay)]
#[template = "templates/foot.html"]
pub struct TemplateFoot;

#[derive(BartDisplay)]
#[template_string = "{{{head}}}{{{body}}}{{{foot}}}"]
pub struct Template<'a, T: Display> {
    head: TemplateHead<'a>,
    body: T,
    foot: TemplateFoot,
}

impl<'a, T: Display> Template<'a, T> {
    pub fn new(title: Option<&'a str>,
               body: T)
        -> Self
    {
        let head = TemplateHead::new(title);

        Template {
            head: head,
            body: body,
            foot: TemplateFoot,
        }
    }
}

#[derive(BartDisplay)]
#[template = "templates/home.html"]
pub struct HomeTmpl {
    pub links: Vec<Link>,
    pub search_uses: (u32, u32),
}

#[derive(BartDisplay)]
#[template = "templates/settings.html"]
pub struct SettingsTmpl {
    pub bangs: Vec<Bang>,
    pub links: Vec<Link>,
    pub api_key: String,
}

#[derive(BartDisplay)]
#[template = "templates/finance/home.html"]
pub struct FinanceHomeTmpl {
    pub accounts: Vec<Account>,
}

#[derive(BartDisplay)]
#[template = "templates/finance/account.html"]
pub struct FinanceAccountTmpl {
    pub transactions: Transactions,
}
