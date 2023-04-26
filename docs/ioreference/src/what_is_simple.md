# What is `SIMPLE`?

`SIMPLE` is a Building Performance Simulation tool
developed with the purpose of modernizing Building Performance Simulation. This includes,
but it really is not limited to, more appropriately **integrating how "people" experience 
and interact with the buildings they use**. There are two main reasons for this.

The first one is that—based on the [research that led to the design and development of 
`SIMPLE`](https://openaccess.wgtn.ac.nz/articles/thesis/Exploring_modelling_and_simulating_the_Feeling_of_Comfort_in_residential_settings/17085467/1)—**Building Performance Simulation tools should be able to, at least** 
(again, there are more requirements) **allow us to perform holistic simulations** (i.e., 
integrating heat, moisture, light and acoustics). The reason for this is that, when 
people get into a room, they just *feel* it. They do not separate—as Software and 
Building Scientists do—the Thermal from the Daylight from the Acoustic domains. So, 
`SIMPLE` aims to enable holistic Building Performance Simulation. This is done by having 
an architecture that allows separating [the source code of different simulation modules](https://github.com/SIMPLE-BuildingSimulation) while allowing communication between 
modules.

The second reason to focus on people is more related to the changing challenges of 
Building Science. One or two decades ago, many of the buildings we lived in were ~~crap~~ 
not very well insulated, they weren't air-tight, their windows weren't sized to balance 
solar heat gains, daylight, and insulaton, and so on. Under these conditions, it made
a lot of sense to focus just on building physics. We did not *really* know how to design
and build healthy, comfortable and Net-Zero energy buildings... but that has changed. 
There are many examples of extremely good buildings today and Software has become
cheaper, more affordable, fast, and easy to use. Today, we understand and can predict the 
effects of building physics better than ever before, which means that **the main gap in 
knowledge lies in human factors**. Insulating an open door is absurd, so we need to 
really understand *when* and *why* people choose to open doors, so we can plan in advance
and design buildings that make their comfort and behaviour compatible with the energy 
efficiency and health needs we have identified.


I like to think of this as a change of paradigm, where we stop focusing on 'occupants' and finally start focusing on 'humans'. If you want to understand a 
bit more the difference I make between Occupants and People [read this](https://buildingsforpeople.org/2020-08-14.blog). 



# Do we need `SIMPLE`? Can't we just keep using other tools?

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

