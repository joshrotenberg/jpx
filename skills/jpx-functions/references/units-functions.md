# Units Functions (4)

## `convert_length`

**Signature:** `number, string, string -> number`

Convert a length value between meters, kilometers, miles, feet, inches, centimeters, millimeters, yards, and nautical miles

```
convert_length(`1`, 'km', 'mi') -> 0.621371
```
_Kilometers to miles_

```
convert_length(`1`, 'ft', 'm') -> 0.3048
```
_Feet to meters_

---

## `convert_mass`

**Signature:** `number, string, string -> number`

Convert a mass value between kilograms, grams, milligrams, pounds, ounces, tonnes, and stones

```
convert_mass(`1`, 'kg', 'lbs') -> 2.20462
```
_Kilograms to pounds_

```
convert_mass(`1`, 'lbs', 'kg') -> 0.453592
```
_Pounds to kilograms_

---

## `convert_temperature`

**Signature:** `number, string, string -> number`

Convert a temperature value between Celsius, Fahrenheit, and Kelvin

```
convert_temperature(`100`, 'C', 'F') -> 212
```
_Boiling point of water in Fahrenheit_

```
convert_temperature(`32`, 'F', 'C') -> 0
```
_Freezing point in Celsius_

---

## `convert_volume`

**Signature:** `number, string, string -> number`

Convert a volume value between liters, milliliters, gallons, quarts, pints, cups, fluid ounces, tablespoons, and teaspoons

```
convert_volume(`1`, 'gal', 'l') -> 3.78541
```
_Gallons to liters_

```
convert_volume(`1`, 'l', 'ml') -> 1000
```
_Liters to milliliters_

---

