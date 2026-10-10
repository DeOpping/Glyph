use crate::namespaced_key::NamespacedKey;

/// Parses input and resolves each valid namespaced placeholder.
///
/// Unrecognized or unresolved placeholders remain unchanged.
pub(super) fn parse(
    input: &str,
    mut resolver: impl FnMut(&NamespacedKey) -> Option<String>
) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;

    while cursor < input.len() {
        let Some(open_offset) = input[cursor..].find('%') else {
            output.push_str(&input[cursor..]);
            break;
        };

        let open = cursor + open_offset;
        output.push_str(&input[cursor..open]);

        let token_start = open + 1;

        let Some(close_offset) = input[token_start..].find('%') else {
            output.push_str(&input[open..]);
            break;
        };

        let close = token_start + close_offset;
        let token = &input[token_start..close];
        let original = &input[open..=close];

        if let Ok(key) = token.parse::<NamespacedKey>() {
            match resolver(&key) {
                Some(replacement) => output.push_str(&replacement),
                None => output.push_str(original)
            }
        } else {
            output.push_str(original);
        }

        cursor = close + 1;
    }

    output
}