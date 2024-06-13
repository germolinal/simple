#!/usr/bin/bash


IMG=./img.hdr
RPICT_OPTIONS="-ab 4 -aa 0.0"
RTRACE_OPTIONS="-lw 1e-10 -ad 10024 -aa 0"
RCONTRIB_OPTIONS="-lw 1e-10 -ad 30000"
POINTS="../../points.pts"
N_BOUNCES=10
WHITE_SKY=sky.rad
OCTREE_GLASS=octree.oct
OCTREE_NO_GLASS=octree_no_glass.oct
BLACK_OCTREE_GLASS=black_$OCTREE_GLASS.oct
BLACK_OCTREE_NO_GLASS=black_$OCTREE_NO_GLASS.oct

# # Run rtrace sims
# cd ray_tracer
# for dir in $(ls -d */)
# do
#     cd $dir
#     rm -rf scene.rad
#     for rad in $(ls | grep .rad)
#     do
#         echo Running sim on $dir

#         oconv -f $rad > $OCTREE_GLASS # Frozen octree

#         # rpict $RPICT_OPTIONS -vp 2 1 1 -vd 0 1 0 -vh 60 -vv 60 -x 512 -y 512 $OCTREE_GLASS > $IMG
#         cat $POINTS | rtrace -h -ab 0 $RTRACE_OPTIONS $OCTREE_GLASS | rcalc -e '$1=$1*0.265 + $2*0.67 + $3*0.065' > direct_results.txt
#         cat $POINTS | rtrace -h -ab $N_BOUNCES $RTRACE_OPTIONS $OCTREE_GLASS | rcalc -e '$1=$1*0.265 + $2*0.67 + $3*0.065' > global_results.txt

#         # echo 2 1 1 0 1 0 | /Users/germolinal/Documents/Radiance/build/UILD_HEADLESS/bin/Debug/rtrace -h /Users/germolinal/Documents/simple/rendering/tests/metal_box_diffuse/octree.oct

#         rm -rf $OCTREE_GLASS


#     done

#     cd ..
# done
# cd ..


# DC sims
cd dc
for dir in $(ls -d */)
do
    cd $dir
    for rad in $(ls | grep .rad)
    do

        # Build scene... for SIMPLE to read afterwards
        cat ./room.rad  > ./scene.rad
        cat ./windows.rad >> ./scene.rad


        oconv -f ./room.rad ./windows.rad > $OCTREE_GLASS # Frozen octree
        oconv -f ./room.rad > $OCTREE_NO_GLASS # Frozen octree

        echo "void plastic black 0 0 5 0 0 0 0 0" > aux_glass
        echo "!xform -m black ./room.rad" >> aux_glass
        echo "!xform ./windows.rad" >> aux_glass
        oconv -f aux_glass > $BLACK_OCTREE_GLASS # Frozen octree

        echo "void plastic black 0 0 5 0 0 0 0 0" > aux_no_glass
        echo "!xform -m black ./room.rad" >> aux_no_glass
        oconv -f aux_no_glass > $BLACK_OCTREE_NO_GLASS # Frozen octree

        echo "#@rfluxmtx u=+Y h=u
            void glow groundglow
            0
            0
            4 1 1 1 0

            groundglow source ground
            0
            0
            4 0 0 -1 180

            #@rfluxmtx u=+Y h=r1
            void glow skyglow
            0
            0
            4 1 1 1 0

            skyglow source skydome
            0
            0
            4 0 0 1 180" > $WHITE_SKY


        N_SENSORS=14


        cat $POINTS | rfluxmtx -y $N_SENSORS -I+ -ab $N_BOUNCES $RCONTRIB_OPTIONS - $WHITE_SKY -i $BLACK_OCTREE_GLASS   > direct_results_glass.txt
        cat $POINTS | rfluxmtx -y $N_SENSORS -I+ -ab $N_BOUNCES $RCONTRIB_OPTIONS - $WHITE_SKY -i $OCTREE_GLASS  > global_results_glass.txt

        cat $POINTS | rfluxmtx -y $N_SENSORS -I+ -ab $N_BOUNCES $RCONTRIB_OPTIONS - $WHITE_SKY -i $BLACK_OCTREE_NO_GLASS   > direct_results_no_glass.txt
        cat $POINTS | rfluxmtx -y $N_SENSORS -I+ -ab $N_BOUNCES $RCONTRIB_OPTIONS - $WHITE_SKY -i $OCTREE_NO_GLASS  > global_results_no_glass.txt

        # Clean up
        rm $WHITE_SKY
        rm -rf $OCTREE_GLASS
        rm -rf $BLACK_OCTREE_GLASS
        rm aux_glass
        rm aux_no_glass
        rm $OCTREE_NO_GLASS
        rm $BLACK_OCTREE_NO_GLASS
    done

    cd ..
done

cd ..
