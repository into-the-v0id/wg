use maud::{html, Markup, PreEscaped};
use wg_core::value::Language;

pub fn default(
    language: &Language,
    title: &str,
    content: Markup,
) -> Markup {
    html! {
        (maud::DOCTYPE)
        html lang=(language) {
            head {
                meta charset="utf-8";
                meta http-equiv="Content-Type" content="text/html; charset=utf-8";
                title { (title) }
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta http-equiv="X-UA-Compatible" content="IE=edge";

                meta name="format-detection" content="telephone=no,date=no,address=no";
                meta name="color-scheme" content="dark light";
                meta name="supported-color-schemes" content="dark light";

                style type="text/css" {
                    (PreEscaped(r#"
                        :root, html, body {
                            width: 100%;
                            -webkit-text-size-adjust: 100%;
                            font-family: system-ui, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, Helvetica, Arial, "Helvetica Neue", sans-serif, "Apple Color Emoji", "Segoe UI Emoji", "Segoe UI Symbol", "Noto Color Emoji";
                            font-weight: 400;
                            line-height: 1.5;
                            background-color: #FFFFFF;
                            color: #373C44;
                            color-scheme: light;
                        }

                        html {
                            margin: 0;
                            padding: 0;
                        }

                        body {
                            box-sizing: border-box;
                            margin: 0;
                            padding: 1em;
                        }

                        a, a:visited {
                            text-decoration: underline;
                            color: #0172AD;
                        }

                        a:hover, a:focus, a:active {
                            text-decoration: underline;
                            color: #015887;
                        }

                        p {
                            margin: 1.5em 0 0.75em 0;
                        }

                        ul, ol {
                            margin: 0.75em 0 1.5em 0;
                        }

                        #header {
                            margin-top: 1em;
                            margin-bottom: 2.75em;
                        }

                        #logo {
                            font-weight: 700;
                            font-size: 1.65em;
                            line-height: 1.125;
                            color: #2D3138;
                        }

                        @media (prefers-color-scheme: dark) {
                            :root, html, body {
                                background-color: #13171F;
                                color: #C2C7D0;
                                color-scheme: dark;
                            }

                            a, a:visited {
                                color: #01AAFF;
                            }

                            a:hover, a:focus, a:active {
                                color: #79C0FF;
                            }

                            #logo {
                                color: #F0F1F3;
                            }
                        }
                    "#))
                }
            }
            body {
                div #header {
                    div #logo { "🏠 WG" }
                }

                div #main {
                    (content)
                }
            }
        }
    }
}
