# JMESPath Filter Patterns

Cookbook of common filter and projection patterns.

## Basic Filtering

### Filter by field value
```
[?status == 'active']
[?type != 'draft']
[?priority == `1`]
```

### Filter by numeric range
```
[?age >= `18` && age <= `65`]
[?price < `100`]
[?score > `90`]
```

### Filter by string content
```
[?contains(name, 'admin')]
[?starts_with(email, 'test')]
[?ends_with(filename, '.json')]
```

### Filter by null/non-null
```
[?email != null]           -- has email
[?deleted_at == null]      -- not deleted
```

### Filter by boolean
```
[?active]                  -- active is truthy
[?!archived]               -- archived is falsy
```

## Combining Conditions

### AND
```
[?status == 'active' && role == 'admin']
[?age >= `18` && country == 'US']
```

### OR
```
[?status == 'error' || status == 'warning']
[?role == 'admin' || role == 'superadmin']
```

### Nested conditions
```
[?(status == 'active' && role == 'admin') || priority == `1`]
```

## Filter + Transform Patterns

### Filter then extract fields
```
[?active].name                                    -- names of active items
[?score > `90`].{student: name, grade: score}     -- reshape high scorers
```

### Filter, sort, take top N
```
[?status == 'published'] | sort_by(@, &date) | reverse(@) | [:10]
```

### Count matches
```
length([?type == 'error'])
```

### Check existence
```
length([?status == 'failed']) > `0`     -- any failures?
length([?status != 'ok']) == `0`        -- all OK?
```

## Nested Data Patterns

### Filter on nested field
```
[?address.country == 'US']
[?config.enabled]
[?metadata.tags[0] == 'important']
```

### Filter nested arrays
```
departments[*].employees[?role == 'engineer']     -- engineers per dept (nested arrays)
departments[*].employees[?role == 'engineer'][]   -- all engineers (flattened)
```

### Filter then navigate deeper
```
[?type == 'order'].items[*].product_id            -- nested arrays per filtered item
[?type == 'order'].items[].product_id             -- flattened product IDs
```

## Projection Patterns

### Extract specific fields from array of objects
```
[*].{id: id, name: name}
```

### Compute derived values
```
[*].{name: name, full: join(' ', [first, last]), age_group: type(age)}
```

### Flatten nested arrays
```
[*].tags[]                 -- all tags from all items
[*].orders[*].items[]      -- all items from all orders
```

### Object to array of entries
```
keys(@)                    -- just keys
values(@)                  -- just values
```

## Aggregation Patterns

### Sum/avg/min/max of projected values
```
[*].price | sum(@)
[*].score | avg(@)
[*].age | max(@)
```

### Group then aggregate (with jpx extensions)
```
group_by(@, &category)     -- group into object by category
```

### Distinct values
```
[*].status | sort(@)       -- sorted (spec only, no unique)
```
