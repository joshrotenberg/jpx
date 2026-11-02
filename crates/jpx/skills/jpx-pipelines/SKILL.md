---
name: jpx-pipelines
description: Slice JSON that other commands print or that sits in files, using jpx in shell pipelines. Use to pull fields, filter, count, or get a yes/no answer from gh, kubectl, curl, or log output, to stream NDJSON, to read plain text lines, and to feed results to shell loops.
---

# jpx in pipelines

Pipe a command's JSON into jpx, or name a file with `-f`. Single-quote the
expression so the shell leaves its backticks and `$` alone.

Run the examples in order in one scratch directory: later ones reuse the files
earlier ones create. Each `text` block shows exactly what the example prints.
`cat` stands in for the real command, which is named in the text.

## Filter and reshape command output

`gh pr list --json number,title,isDraft,author` prints an array like this one:

```sh
cat > prs.json <<'JSON'
[{"number":41,"title":"Fix parser panic","isDraft":false,"author":{"login":"ada"}},
 {"number":42,"title":"Add CSV output","isDraft":true,"author":{"login":"lin"}},
 {"number":43,"title":"Bump serde","isDraft":false,"author":{"login":"ada"}}]
JSON
cat prs.json | jpx -c -S '[?!isDraft].{number: number, author: author.login}'
cat prs.json | jpx -c 'count_by(&author.login, @)'
```

```text
[{"author":"ada","number":41},{"author":"ada","number":43}]
{"ada":2,"lin":1}
```

`-c` prints compact JSON. Object key order can differ between jpx builds, so
add `-S` to sort keys when output is compared or diffed. A null result prints
nothing; use `-x` when the shell needs to know.

Nested output, such as `kubectl get pods -o json`, works the same way:

```sh
cat > pods.json <<'JSON'
{"items":[
 {"metadata":{"name":"api-7d9"},"status":{"phase":"Running"}},
 {"metadata":{"name":"worker-2x1"},"status":{"phase":"Pending"}}]}
JSON
jpx -c 'items[?status.phase != `"Running"`].metadata.name' -f pods.json
```

```text
["worker-2x1"]
```

## Feed shell loops and conditionals

`-l` prints one array element per line, and `-r` drops the quotes from
strings:

```sh
jpx -l -r '[?!isDraft].title' -f prs.json | while read -r title; do
  echo "ready: $title"
done
```

```text
ready: Fix parser panic
ready: Bump serde
```

`-x` sets the exit status from the result: 0 when it is truthy, 1 when it is
`false` or null. Pass shell values with `--arg`; `$who` inside the single
quotes is a jpx variable, not shell expansion:

```sh
if jpx -x 'length([?isDraft]) > `0`' -f prs.json > /dev/null; then
  echo "drafts open"
fi
who=ada
jpx -c --arg who "$who" '[?author.login == $who].number' -f prs.json
```

```text
drafts open
[41,43]
```

## Stream NDJSON

Logs and `--paginate`-style output often print one JSON object per line.
`--stream` evaluates each line on its own with constant memory and skips null
results. `-s` instead reads every line into one array, for totals:

```sh
printf '%s\n' '{"level":"info","msg":"start","ms":12}' \
  '{"level":"error","msg":"timeout","ms":5003}' \
  '{"level":"error","msg":"refused","ms":3}' > app.log
jpx --stream -c -S 'level == `"error"` && {msg: msg, ms: ms} || `null`' < app.log
jpx -s 'length([?level == `"error"`])' < app.log
```

```text
{"ms":5003,"msg":"timeout"}
{"ms":3,"msg":"refused"}
2
```

## Plain text, several files, and JSON inside strings

`-R` reads each input line as a string, and `-R -s` makes one array of them,
for output such as `git branch --format='%(refname:short)'`. `--per-file`
evaluates the expression once per file and prints one array of results.
`json_decode` parses a field that holds JSON text, as some API fields do:

```sh
printf 'main\nfeat/csv\nfix/panic\nfeat/skills\n' |
  jpx -R -s -c '[?starts_with(@, `"feat/"`)]'
echo '[{"number":7,"title":"Docs","isDraft":true,"author":{"login":"kim"}}]' > more.json
jpx -c --per-file 'length(@)' prs.json more.json
printf '%s\n' '{"events":[{"payload":"{\"ref\":\"main\"}"}]}' |
  jpx -c 'events[].json_decode(payload).ref'
```

```text
["feat/csv","feat/skills"]
[3,1]
["main"]
```

## Save this skill

To keep a local copy, run from the directory where you want a `skills` folder:

```sh
mkdir -p skills/jpx-pipelines
jpx --skills pipelines > skills/jpx-pipelines/SKILL.md
```

The output is this entire standalone skill, including YAML frontmatter. Place
the `jpx-pipelines` folder in your agent's supported skill directory to install
it. No companion files or network access are required.
