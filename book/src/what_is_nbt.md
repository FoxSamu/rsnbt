# What is NBT
**Named Binary Tag**, **NBT** for short, is a binary data format used by Minecraft to store and exchange information. NBT can be more or less seen as a binary alternative to JSON. It easily encodes and decodes, reducing any formatting and parsing overhead that JSON would introduce. It is thus a very effective format for data exchange and storage, though it comes at the disadvantage that it is not human-readable (in the sense that you cannot just open an NBT file in a text editor and see immediately what is going on, though you can often inspect them using special tools).

NBT is used extensively by _Minecraft: Java Edition_, and in lesser amounts also by _Minecraft: Bedrock Edition_. In this book we mainly focus on _Java Edition_.

## Tags
At the very fundamental level, every data object packaged in NBT format is packaged as a **tag**. Every tag has a _tag ID_, a _name_ and a _payload_. One NBT file or message typically encodes one single tag, though there are possibilities to encode tags inside other tags, that we'll see in a bit.

The **tag ID** identifies the type of tag using a small identifier. This hints at a very important detail about NBT tags: there are different types of tags. The tag ID marks exactly what type a tag is, so that anyone or anything that wants to decode the NBT knows exactly what the data is going to be structured like and what they can expect.

The **name** is the second part of a tag, and it makes the tag identifiable when grouped with other tags. In NBT, every tag has a name. Even when you don't group a tag with any others, the tag must have a name, despite it not being any relevant in most cases. Minecraft often sets irrelevant names to be empty (a name of zero characters long).

The **payload** is the final part of a tag, and this is where the actual data of the tag is structured. You can see the tag ID and the name as a sort of _header_ for a tag, and the payload as its _content_. The way the payload is structured is determined by the tag ID.

### Types
As we just discussed, the tag ID tells us exactly how a tag's payload is structured. This tag ID is a number (more specifically, it's a single byte), and a different number means a different format. We'll see each tag in more detail soon, but here's an overview.

| ID     | Tag Type   | Description                                                                                 |
|:------:|------------|---------------------------------------------------------------------------------------------|
|    0   | End        | A special end marker, we'll see the meaning of this in a bit.                               |
|    1   | Byte       | One byte of data, often a small number or a boolean value.                                  |
|    2   | Short      | Two bytes of data, usually a small-ish number.                                              |
|    3   | Int        | Four bytes of data, usually a number.                                                       |
|    4   | Long       | Eight bytes of data, usually a very large number.                                           |
|    5   | Float      | An IEEE 754 floating point value, of 32 bits (4 bytes).                                     |
|    6   | Double     | A double precision IEEE 754 floating point value, of 64 bits (8 bytes).                     |
|    7   | Byte Array | An array of bytes, of arbitrary length. The payload includes length information.            |
|    8   | String     | An UTF-8 encoded string. Again, length is included in the payload.                          |
|    9   | List       | A list of payloads. Each payload is the same type, which the first payload byte determines. |
|   10   | Compound   | A collection of full tags, each with an ID, name and their own payloads.                    |
|   11   | Int Array  | An array of Int values (each 4 bytes).                                                      |
|   12   | Long Array | An array of Long values (each 8 bytes). Not supported by old versions of Minecraft (<1.13). |
|   13+  | Reserved   | These IDs are unused.                                                                       |

### Primitives
When we mention **primitive tags** or **primitives**, what we mean is any tag with a type ID from 1 to 6 (inclusive). Those are: _Byte_, _Short_, _Int_, _Long_, _Float_ and _Double_. All of these are numeric values, and typically represent numbers, though other than Float and Double, they are also often used to store more compressed data. The primitives are named after their equivalent types types in Java: `byte`, `short`, `int`, `long`, `float` and `double`. If you know what each of these types is in Java, then you know exactly what the payloads of these tags look like.

The **Byte**, **Short**, **Int** and **Long** tags store integers. More specifically, they store _signed_ integers. To put it simply, it means you can slap a `-` in front of them and that would also be a valid value. Integers have the limitation that they are all restricted to a range. After all, you can only store so much different numbers in a fixed amount of bits.

| Type  | Minimum (inclusive)               | Maximum (inclusive)               |
|-------|-----------------------------------|-----------------------------------|
| Byte  | -128                              | 127                               |
| Short | -32,768                           | 32,767                            |
| Int   | -2,147,483,648                    | 2,147,483,647                     |
| Long  | -9,223,372,036,854,775,808        | 9,223,372,036,854,775,807         |

The **Float** and **Double** tags also store numbers, but they work a little differently. Most importantly, they can store fractional numbers, e.g. 0.5 or 3.1415. We call these numbers _floating point numbers_, _floats_ for short. Again, there are limitations, but they are different from integers. We'll not cover floating point numbers in excruciating detail here, but here's a short crash course:

- floats consist of a signed, fixed-precision _mantissa_, paired with a _magnitude_ or _exponent_, much like scientific notation;
    - e.g. 11400 can also be written as 1.14 &times; 10<sup>4</sup>, where 1.14 is the mantissa and 4 is the magnitude;
    - similarly, in binary, 1001011<sub>2</sub> can be written as 1.001011<sub>2</sub> &times; 2<sup>6</sup>;
- floats have no direct numeric limit, they go from negative infinity to positive infinity;
- the magnitude has a lower and upper bound: below the lower bound the value is considered 0, and above the upper bound the value is considered _infinity_;
- more bits means more precise floating point numbers and larger magnitude range;
- large float values are less accurate than small float values;
- some float values are invalid and are called _not a number_: _NaN_;
- negative zero and positive zero are logically equivalent but are different values;
- pretty much all uses of floats nowadays follow the [IEEE 754 standard](https://en.wikipedia.org/wiki/IEEE_754).

Good, now you're up to date on what floats are. NBT allows storing _single-precision_ (32-bits) floats and _double precision_ (64-bits) floats, using respectively the _Float_ tag type and _Double_ tag type. Like any other proper supporter of floats, NBT stores them according to the [IEEE 754 standard](https://en.wikipedia.org/wiki/IEEE_754). This means that we can have values like Infinity, -0 and NaN in NBT.

<!-- TODO float table -->