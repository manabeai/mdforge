use super::Forge;

pub(super) fn build_signature(forge: &Forge) -> String {
    let mut lines = Vec::new();

    for block in &forge.blocks {
        lines.push(format!("Block: {}", block.name));
        let args = block
            .args
            .iter()
            .map(|(name, spec)| {
                format!("{}={}", name, spec.arg_type.signature_label(spec.required))
            })
            .collect::<Vec<_>>()
            .join(" ");

        let head = if args.is_empty() {
            format!(":::{}", block.name)
        } else {
            format!(":::{} {}", block.name, args)
        };
        lines.push(head);
        if block.body_markdown {
            lines.push("Body: markdown".to_string());
        }
        lines.push(String::new());
    }

    for inline in &forge.inlines {
        lines.push(format!("Inline: {}", inline.name));
        let args = inline
            .args
            .iter()
            .map(|(name, spec)| {
                format!("{}={}", name, spec.arg_type.signature_label(spec.required))
            })
            .collect::<Vec<_>>()
            .join(" ");
        let body = if args.is_empty() {
            format!("{{{}}}", inline.name)
        } else {
            format!("{{{} {}}}", inline.name, args)
        };
        lines.push(body);
        lines.push(String::new());
    }

    while matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }

    lines.join("\n")
}
