


```

>

[x] [x] [x] [x]
[x] [x] [x] [x]
[x] [x] [x] [x]
[x] [x] [x] [x]

>

>> if below & to the left & right, that's support
>> ...

[ ] [x] [x] [x] [x] [x] [x]
[ ] [-] [x] [x] [x] [x] [x]
[ ] [ ] [ ] [ ] [ ] [ ] [x]
[x] [x] [x] [x] [x] [x] [x]
[x] [x] [x] [x] [x] [x] [x]

>

[x] [x] [ ] [x] [x] [x]
[x] [x] [-] [x] [x] [x]
[ ] [ ] [ ] [ ] [ ] [x]
[x] [x] [x] [x] [x] [x]

?? how do you shore things up? Maybe make a "platform". Yeah that would be good.

Hmm nphysics platform....
hmm what if I have a collider at the bottom, and if you go up through the platform then it's disabled until you pass through?

```


....
umm seems like you'll need a bunch of supports if I'm going to do this that way.
I think a "plank" will probably serve as a support just fine.
Oh, but what if the plank can break sometimes? That would be interesting.


Things that could trigger cave-ins:
- digging down, if there's only one block below you (you can fall down!), very short fuze on that
- a bottom "corner" will fall if you don't have a support or a door under it, but it'll take a minute or two
- if a block is on the bottom, and more than ~4 (TBD exact count) from a supported block, then it'll fall down after bit.


How cave-ins work:
- the block shows cracks for like 1 second, and then gives out. so you have barely a bit of warning.
  umm maybe I'll randomize it.


More thoughts about crafting -- if you build a large thing (like a support maybe), then I don't think it should be able to fit in your backpack. It should be "build on site". And then you can deconstruct. But when deconstructing, based on your crafting skill, you won't recover all items (until you're very good at crafting).

Also for deconstructing, you need a tool (sharp rock will do), and probably the quality of the tool impacts how much you can recover.

When crafting a support thing, if it's up to 4 blocks high, then you use one plank for each block of height.

