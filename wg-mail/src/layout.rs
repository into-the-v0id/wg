use maud::{html, Markup};

pub fn default(
    title: &str,
    content: Markup,
) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (title) }
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta http-equiv="Content-Type" content="text/html; charset=UTF-8";

                style media="all" type="text/css" {
                    r#"
                        body {
                            width: 100%;
                            height: 100%;
                            box-sizing: border-box;
                            padding: 1em;
                            margin: 0;
                        }
                    "#
                }
            }
            body {
                (content)
            }
        }
    }
}
