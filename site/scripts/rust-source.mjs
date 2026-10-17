// Extracts the source of a gallery example from the Rust file that builds it,
// with what it needs to read on its own: the `use` declarations, the helper
// functions and constants of the file, and the `let` bindings of the view
// that it refers to.

/** Splits Rust source into tokens (comments and whitespace included). */
export function tokenize(src) {
  const tokens = [];
  let i = 0;

  const push = (type, end) => {
    tokens.push({ type, text: src.slice(i, end), start: i, end });
    i = end;
  };

  while (i < src.length) {
    const c = src[i];
    const rest = src.slice(i, i + 8);

    if (/\s/.test(c)) {
      let j = i;
      while (j < src.length && /\s/.test(src[j])) j++;
      push("ws", j);
    } else if (rest.startsWith("//")) {
      let j = src.indexOf("\n", i);
      if (j === -1) j = src.length;
      push("comment", j);
    } else if (rest.startsWith("/*")) {
      let depth = 0;
      let j = i;
      do {
        if (src.startsWith("/*", j)) {
          depth++;
          j += 2;
        } else if (src.startsWith("*/", j)) {
          depth--;
          j += 2;
        } else {
          j++;
        }
      } while (depth > 0 && j < src.length);
      push("comment", j);
    } else if (/^b?r#*"/.test(rest)) {
      const [prefix] = rest.match(/^b?r(#*)"/);
      const hashes = prefix.replace(/^b?r/, "").replace(/"$/, "");
      const close = `"${hashes}`;
      const j = src.indexOf(close, i + prefix.length);
      push("string", j === -1 ? src.length : j + close.length);
    } else if (c === '"' || rest.startsWith('b"')) {
      let j = i + (c === '"' ? 1 : 2);
      while (j < src.length && src[j] !== '"') j += src[j] === "\\" ? 2 : 1;
      push("string", j + 1);
    } else if (c === "'" || rest.startsWith("b'")) {
      const offset = c === "'" ? 0 : 1;
      const char = src.slice(i + offset, i + offset + 12).match(/^'(\\u\{[0-9a-fA-F]+\}|\\.|[^\\'\n])'/);
      if (char) {
        push("char", i + offset + char[0].length);
      } else {
        const lifetime = src.slice(i).match(/^'[A-Za-z_]\w*/);
        push("lifetime", i + (lifetime ? lifetime[0].length : 1));
      }
    } else if (/[A-Za-z_]/.test(c)) {
      const [ident] = src.slice(i).match(/^(r#)?[A-Za-z_]\w*/);
      push("ident", i + ident.length);
    } else if (/\d/.test(c)) {
      const [number] = src.slice(i).match(/^\d[\w]*(\.\d[\w]*)?/);
      push("number", i + number.length);
    } else {
      push("punct", i + 1);
    }
  }

  return tokens;
}

const OPEN = { "(": ")", "[": "]", "{": "}" };

/** Parses the tree of a `use` declaration from its tokens (no whitespace). */
function parseUse(tokens) {
  let k = 0;

  function tree() {
    const path = [];
    let children = null;
    let alias = null;

    for (;;) {
      const t = tokens[k];
      if (t?.text === "{") {
        k++;
        children = [];
        while (tokens[k] && tokens[k].text !== "}") {
          children.push(tree());
          if (tokens[k]?.text === ",") k++;
        }
        k++;
        break;
      } else if (t?.type === "ident" || t?.text === "*") {
        path.push(t.text);
        k++;
        if (tokens[k]?.text === ":" && tokens[k + 1]?.text === ":") {
          k += 2;
          continue;
        }
        break;
      } else {
        break;
      }
    }

    if (tokens[k]?.text === "as") {
      alias = tokens[k + 1].text;
      k += 2;
    }

    return { path, children, alias };
  }

  return tree();
}

/** Keeps the parts of a `use` tree that bring a name of `used` in scope. */
function pruneUse(tree, used, parent = null) {
  if (tree.children) {
    const children = tree.children.map((child) => pruneUse(child, used, tree.path.at(-1) ?? parent)).filter(Boolean);
    return children.length ? { ...tree, children } : null;
  }

  const last = tree.path.at(-1);
  if (last === "*") return tree;
  const name = tree.alias ?? (last === "self" ? (tree.path.at(-2) ?? parent) : last);
  return used.has(name) ? tree : null;
}

function printUse(tree) {
  let path = tree.path.join("::");

  if (tree.children) {
    const children = tree.children.map(printUse);
    const nested =
      children.length === 1 && tree.children[0].path[0] !== "self" ? children[0] : `{${children.join(", ")}}`;
    path = path ? `${path}::${nested}` : nested;
  }

  return tree.alias ? `${path} as ${tree.alias}` : path;
}

/** The names a `use` tree brings in scope. */
function useNames(tree, names = new Set(), parent = null) {
  if (tree.children) {
    for (const child of tree.children) useNames(child, names, tree.path.at(-1) ?? parent);
  } else {
    const last = tree.path.at(-1);
    if (last !== "*") {
      names.add(tree.alias ?? (last === "self" ? (tree.path.at(-2) ?? parent) : last));
    }
  }
  return names;
}

/** The `use` declaration of `tokens[from..to]`, pruned to `used`. */
function useDeclaration(tokens, from, to, used) {
  const code = tokens.slice(from, to).filter((t) => t.type !== "ws" && t.type !== "comment");
  const tree = pruneUse(parseUse(code.slice(1, -1)), used);
  if (!tree) return null;
  // The gallery pages are `crate::pages::*`.
  return `use ${printUse(tree)};`.replace(/^use super::/, "use crate::pages::");
}
const CLOSE = new Set([")", "]", "}"]);

/** Removes the common indentation of `text`. */
function dedent(text) {
  const lines = text.replace(/\s+$/, "").split("\n");
  const indent = Math.min(...lines.filter((line) => line.trim()).map((line) => line.match(/^ */)[0].length));
  return lines.map((line) => line.slice(indent)).join("\n");
}

/** The source from the start of the line of `start` to `end`, dedented. */
function block(src, start, end) {
  const lineStart = src.lastIndexOf("\n", start - 1) + 1;
  const prefix = src.slice(lineStart, start);
  return dedent((/^\s*$/.test(prefix) ? prefix : " ".repeat(prefix.length)) + src.slice(start, end));
}

/** The identifiers that `tokens[from..to]` refers to. */
function identifiers(tokens, from, to) {
  const names = new Set();
  for (let k = from; k < to; k++) {
    if (tokens[k].type === "ident") names.add(tokens[k].text);
  }
  return names;
}

/** The index of the next token that is not whitespace or a comment. */
function next(tokens, k) {
  while (k < tokens.length && (tokens[k].type === "ws" || tokens[k].type === "comment")) k++;
  return k;
}

/**
 * Parses the items of a file (its top level): `use` declarations, functions,
 * constants and statics, with their doc comments and attributes.
 */
function items(src, tokens) {
  const result = [];
  let depth = 0;
  let itemStart = null;

  for (let k = 0; k < tokens.length; k++) {
    const token = tokens[k];

    if (depth === 0 && itemStart === null && token.type !== "ws") {
      itemStart = k;
    }

    if (token.type === "punct" && OPEN[token.text]) depth++;
    if (token.type === "punct" && CLOSE.has(token.text)) {
      depth--;
      if (depth === 0 && token.text === "}" && itemStart !== null) {
        const after = next(tokens, k + 1);
        // `Page { … };` items end with a semicolon.
        if (tokens[after]?.text !== ";") {
          result.push(item(src, tokens, itemStart, k + 1));
          itemStart = null;
        }
      }
    }
    if (depth === 0 && token.type === "punct" && token.text === ";" && itemStart !== null) {
      result.push(item(src, tokens, itemStart, k + 1));
      itemStart = null;
    }
    if (depth === 0 && token.type === "comment" && itemStart === k && !token.text.startsWith("///")) {
      // Plain comments between items do not start an item.
      itemStart = null;
    }
  }

  return result.filter(Boolean);
}

function item(src, tokens, from, to) {
  let k = from;
  // Skip doc comments, attributes and visibility to find the kind of item.
  for (;;) {
    k = next(tokens, k);
    if (tokens[k]?.text === "#") {
      let depth = 0;
      for (k++; k < to; k++) {
        if (tokens[k].text === "[") depth++;
        if (tokens[k].text === "]" && --depth === 0) break;
      }
      k++;
    } else if (tokens[k]?.text === "pub") {
      k = next(tokens, k + 1);
      if (tokens[k]?.text === "(") {
        while (tokens[k].text !== ")") k++;
        k++;
      }
    } else {
      break;
    }
  }

  const kind = tokens[k]?.text;
  if (!["use", "fn", "const", "static"].includes(kind)) return null;

  const name = kind === "use" ? null : tokens[next(tokens, k + 1)]?.text;
  const start = tokens[next(tokens, from)].start;
  // Keep the doc comment of the item.
  let docStart = from;
  while (docStart < to && tokens[docStart].type === "ws") docStart++;

  return {
    kind,
    name,
    text: block(src, tokens[docStart].start ?? start, tokens[to - 1].end),
    from,
    to,
    uses: identifiers(tokens, k + 1, to),
    useFrom: k,
    // The names a `use` declaration brings in scope.
    provides:
      kind === "use"
        ? useNames(parseUse(tokens.slice(k + 1, to - 1).filter((t) => t.type !== "ws" && t.type !== "comment")))
        : new Set([name]),
  };
}

/** The `let` statements of the block opening at `tokens[open]`, before `until`. */
function lets(src, tokens, open, until) {
  const result = [];
  let depth = 0;

  for (let k = open + 1; k < until; k++) {
    const token = tokens[k];

    if (token.type === "punct" && OPEN[token.text]) depth++;
    if (token.type === "punct" && CLOSE.has(token.text)) depth--;

    if (depth === 0 && token.type === "ident" && token.text === "use") {
      let end = k + 1;
      while (end < until && tokens[end].text !== ";") end++;
      if (end >= until) break;
      const code = tokens.slice(k + 1, end).filter((t) => t.type !== "ws" && t.type !== "comment");
      result.push({ use: true, names: useNames(parseUse(code)), uses: new Set(), from: k, to: end + 1, index: k });
      k = end;
      continue;
    }

    if (depth === 0 && token.type === "ident" && token.text === "let") {
      let end = k + 1;
      let inner = 0;
      let equals = null;
      for (; end < tokens.length; end++) {
        const t = tokens[end];
        if (t.type === "punct" && OPEN[t.text]) inner++;
        if (t.type === "punct" && CLOSE.has(t.text)) inner--;
        if (inner === 0 && t.text === "=" && equals === null && tokens[end + 1]?.text !== "=") {
          equals = end;
        }
        if (inner === 0 && t.text === ";") break;
      }
      if (end >= until) break;

      const pattern = [...identifiers(tokens, k + 1, equals ?? end)].filter(
        (ident) => !["mut", "ref"].includes(ident) && /^[a-z_]/.test(ident),
      );

      // Leading comments belong to the statement.
      let start = k;
      while (start - 1 > open && ["ws", "comment"].includes(tokens[start - 1].type)) start--;
      while (tokens[start].type === "ws") start++;

      result.push({
        names: new Set(pattern),
        uses: identifiers(tokens, (equals ?? k) + 1, end),
        text: block(src, tokens[start].start, tokens[end].end),
        index: k,
      });
      k = end;
    }
  }

  return result;
}

/**
 * The source of the example built by the `example(…)` call at `line` and
 * `column` (1-based) of `src`.
 */
export function exampleSource(src, line, column) {
  const tokens = tokenize(src);
  const lineStart = src
    .split("\n")
    .slice(0, line - 1)
    .reduce((n, l) => n + l.length + 1, 0);
  const offset = lineStart + column - 1;

  const at = tokens.findIndex((t) => t.start === offset);
  if (at === -1 || tokens[at].text !== "example") {
    throw new Error(`no \`example(\` call at ${line}:${column}`);
  }

  // The blocks around the call, innermost last.
  const stack = [];
  for (let k = 0; k < at; k++) {
    const t = tokens[k];
    if (t.type === "punct" && OPEN[t.text]) stack.push(k);
    if (t.type === "punct" && CLOSE.has(t.text)) stack.pop();
  }

  // `example(` "title" `,` content `)`
  let k = next(tokens, at + 1); // (
  k = next(tokens, k + 1); // title
  k = next(tokens, k + 1); // ,
  const first = next(tokens, k + 1);

  let depth = 0;
  let last = first;
  for (k = first; k < tokens.length; k++) {
    const t = tokens[k];
    if (t.type === "punct" && OPEN[t.text]) depth++;
    if (t.type === "punct" && CLOSE.has(t.text)) {
      if (depth === 0) break;
      depth--;
    }
    if (depth === 0 && t.text === "," && tokens[next(tokens, k + 1)]?.text === ")") break;
    if (t.type !== "ws" && t.type !== "comment") last = k;
  }

  const content = block(src, tokens[first].start, tokens[last].end);
  let used = identifiers(tokens, first, last + 1);

  // The `let` bindings it needs, from the innermost block outwards.
  const bindings = [];
  for (const open of stack.filter((index) => tokens[index].text === "{").toReversed()) {
    bindings.unshift(...lets(src, tokens, open, at));
  }

  const needed = new Set();
  let changed = true;
  while (changed) {
    changed = false;
    for (const binding of bindings) {
      if (!needed.has(binding) && [...binding.names].some((name) => used.has(name))) {
        needed.add(binding);
        used = new Set([...used, ...binding.uses]);
        changed = true;
      }
    }
  }

  // The helper functions and constants of the file it needs.
  const fileItems = items(src, tokens);
  const enclosing = fileItems.find((i) => i.from <= at && at < i.to);
  const helpers = new Set();
  changed = true;
  while (changed) {
    changed = false;
    for (const i of fileItems) {
      if (
        (i.kind === "fn" || i.kind === "const" || i.kind === "static") &&
        i !== enclosing &&
        !helpers.has(i) &&
        used.has(i.name) &&
        // Long constants (data) would drown the example.
        (i.kind === "fn" || i.text.split("\n").length <= 12)
      ) {
        helpers.add(i);
        used = new Set([...used, ...i.uses]);
        changed = true;
      }
    }
  }

  const imports = [
    ...fileItems.filter((i) => i.kind === "use").map((i) => useDeclaration(tokens, i.useFrom, i.to, used)),
    ...bindings.filter((b) => b.use).map((b) => useDeclaration(tokens, b.from, b.to, used)),
  ].filter(Boolean);

  const parts = [];
  if (imports.length) parts.push([...new Set(imports)].join("\n"));
  for (const i of fileItems) if (helpers.has(i)) parts.push(i.text);
  const body = bindings.filter((b) => !b.use && needed.has(b)).map((b) => b.text);
  parts.push([...body, content].join("\n"));

  return { code: parts.join("\n\n"), content };
}
