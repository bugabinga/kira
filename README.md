# kira

> evil light

`kira` is a script, that disguises itself as an rust application, that can set
the brightness of your monitor of your Linux laptop.

The distinguishing feature of `kira`, the reason I wrote it instead of using
alternatives like [light] or [acpilight], which are better in every other
respect, is the smooth setting of the brightness.

Brightness is set step-wise over time resulting in a linear and smooth
transition of brightness.

[light]: https://haikarainen.github.io/light
[acpilight]: https://gitlab.com/wavexx/acpilight

## Usage

```sh
$ kira            # set brightness to 100%
$ kira 33         # set brightness to 33%
$ kira +12        # increase brightness by 10%
$ kira -43        # decrease brightness by 43%
```

## Limitations

- Works only on specific machines and OSes
  - Linux (sysfs)
  - Intel graphics card
- No meaningful error handling
- Interpolation is linear only

## How it works

`kira` writes to the **sysfs** kernel API of Linux.
Specifically, `/sys/class/backlight/intel_backlight`.

The transition is achieved by looping from the current value until the target
value and sleeping the thread for a couple of nanoseconds on each step.
This means the transition speed (smoothness) is dependent on the range of values
of *my specific device* and *magic amount of nanoseconds* I chose.

This happens to work out on my *Thinkpad x230*, but is unlikely to work
universally.

## Working together

If for some bizarre reason you have to urge to collaborate in this project, shoot
me a [mail](mailto:oliver@bugabinga.net) or open an issue to start discussing ideas.
