
pub fn GetArcToSegmentCount(aRadius: f64, aErrorMax: u32, aArcAngleDegree: f64) -> u32 {
  /* std::cout << "arc to segment count: " << aRadius << " " << aErrorMax << " " << aArcAngleDegree << "\n";
    // calculate the number of segments to approximate a circle by segments
    // given the max distance between the middle of a segment and the circle

    // avoid divide-by-zero
    aRadius = std::max( 1, aRadius );

    // error relative to the radius value:
    double rel_error = (double)aErrorMax / aRadius;
    // minimal arc increment in degrees:
    double arc_increment = 180 / M_PI * acos( 1.0 - rel_error ) * 2;

    // Ensure a minimal arc increment reasonable value for a circle
    // (360.0 degrees). For very small radius values, this is mandatory.
    arc_increment = std::min( 360.0/MIN_SEGCOUNT_FOR_CIRCLE, arc_increment );

    int segCount = KiROUND( fabs( aArcAngleDegree ) / arc_increment );

    // Ensure at least two segments are used for algorithmic safety
    std::cout << "== " << std::max( segCount, 2 ) << "\n"; 
    return std::max( segCount, 2 ); */
    0
}
