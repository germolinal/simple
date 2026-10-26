# Can't we just improve other tools?

I have been working in Building Simulation Tool space for nearly a decade now. During
this time, I have tried to combine simulation tools and methods, and also to make
simulation more accessible to people. For example, my MSc research focused on the
integration of Radiance and EnergyPlus. It was then when I started developing [Groundhog](www.groundhoglighting.com), which intended to put Radiance in everyone’s hands. While
developing Groundhog I realized that Radiance could use some sort of wrapper to make it a
bit friendlier by offering out-of-the-box optimizations and pre- and post-processing.
This motivated me to develop Emp, a Radiance orchestrator. Both projects—Radiance and
Emp—have already died for various reasons.

During my time developing building performance simulation tools I have learned about an
enormous amount of undeniable virtues that currently available tools—e.g., EnergyPlus and
Radiance—have: they have been extensively validated, they are open source, they are free
(as in freedom), and so on. Unfortunately, I have also seen many drawbacks. For starters,
these tools were not really meant to be embedded on other tools. Radiance confesses this
quite clearly:

> “These routines are designed to aid the programmer who wishes to call Radiance as a
library.  Unfortunately, the system was not originally intended to be run this way, and
there are some awkward limitations to contend with…” (Radiance’s raycalls.c)


Also, these tools were developed decades ago. Radiance was developed before the gaming
and animation industry invested millions in improving rendering techniques, and
EnergyPlus tends to focus too much on building physics even if we now know that we need
to pay more attention to people’s behaviour in buildings (because insulating an open door
is pointless). Also, programming was different back then, with different source control
systems and way less people who knew how to code.

`SIMPLE` attempts to fix these issues by being open-source from day zero, by using a
modern programming language, by having an architeture designed for enabling
collaboration. Don’t be mistaken, though, `SIMPLE` does not attempt to reinvent the
wheel. The real wheel is the knowledge of physics and rendering and computer sciences
behind currently existing Building Performance Simulation tools… what it proposes is to
build a good-old wheel with new materials and methods, in a more modern factory, and that
is designed to travel in modern roads.
