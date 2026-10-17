container(content).style(|theme: &Theme| {
    let t = tokens(theme);
    container::Style {
        background: Some(t.palette.muted.into()),
        border: style::rounded(t.radius_lg()),
        ..Default::default()
    }
})
