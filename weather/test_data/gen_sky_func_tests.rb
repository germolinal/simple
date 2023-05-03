
@is_solar = true;

def write_test(lat, lon, standard_mer, month, day, hour, direct_normal, diffuse_horizontal, dx, dy, dz)
    dew_point = 11.0;
    len = Math::sqrt(dx*dx + dy*dy + dz*dz)
    dx /= len
    dy /= len
    dz /= len


    puts "let solar = Solar::new(#{lat * Math::PI / 180}, #{lon * Math::PI / 180}, #{standard_mer * Math::PI / 180});"
    puts "let date = Date{month: #{month}, day: #{day}, hour: #{hour}};"
    puts "let dew_point = #{dew_point};"
    puts "let direct_normal_irrad = #{direct_normal};"
    puts "let diffuse_horizontal_irrad = #{diffuse_horizontal};"
    puts "let fun = PerezSky::get_sky_func_standard_time(SkyUnits::#{@is_solar ? "Solar" : "Visible"}, &solar, date, dew_point, diffuse_horizontal_irrad, direct_normal_irrad);"
    puts "let found = fun(Vector3D::new(#{dx}, #{dy}, #{dz}));"
    command = "gendaylit #{month} #{day} #{hour} -W #{direct_normal} #{diffuse_horizontal} -a #{lat} -o #{lon} -m #{standard_mer} #{ @is_solar ? "-O 1" : "-O 0" } | tail -n 1 | rcalc  -e 'A1=$2; A2=$3; A3=$4; A4=$5; A5=$6; A6=$7; A7=$8; A8=$9; A9=$10; A10=$11; Dx=#{dx};  Dy=#{dy}; Dz=#{dz}' -f ./perezlum.cal -o '${intersky}'"
    puts "// Automatically generated using command: #{command}"
    puts "let expected = #{`#{command}`.to_f};"
    puts "println!(\"{}, {}, {} ({}%)\", expected, found, (expected - found).abs(), 100.*(expected - found).abs()/expected);"
    puts "assert!( 100.*(expected - found).abs()/expected < 3. );"
    puts "\n"
end



lats = [1, -33, -47, 47];
lons = [1, -40, 47, 12];
standard_mers = lons;

months = [2, 4, 11, 1];
days = [2, 5, 7, 1];
hours = [12.0, 10.0, 16.0, 13.0];

direct_normals = [500.0, 600.0, 900.0, 300.0];
diffuse_horizontals = [200.0, 200.0, 100.0, 300.0];

dxs = [1.0, 0.0, -3.0]
dys = [2.0, -2.0, 0.0]
dzs = [23.0, 0.1, 0.4]

for i in 0..lats.length-1 do
    lat = lats[i];
    lon = lons[i];
    standard_mer = standard_mers[i];

    month = months[i];
    day = days[i];
    hour = hours[i];

    direct_normal = direct_normals[i];
    diffuse_horizontal = diffuse_horizontals[i];

    for j in 0..dxs.length-1 do
        dx = dxs[j]
        dy = dys[j]
        dz = dzs[j]
    
        write_test(lat, lon, standard_mer, month, day, hour, direct_normal, diffuse_horizontal, dx, dy, dz)
    end
end
