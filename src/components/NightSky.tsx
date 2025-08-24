
import { useMemo } from "react";

const STAR_COUNT = 60;

const NightSky = () => {
  // Generate stars only once
  const stars = useMemo(
    () =>
      Array.from({ length: STAR_COUNT }).map(() => ({
        width: Math.random() * 2 + 1,
        height: Math.random() * 2 + 1,
        top: Math.random() * 100,
        left: Math.random() * 100,
      })),
    []
  );

  return (
    <div className="absolute inset-0 -z-10 bg-gradient-to-b from-[#0a1026] to-[#1a223f] overflow-hidden">
      {/* Moon */}
      <div
        className="absolute top-8 right-16 w-24 h-24 bg-yellow-200 rounded-full shadow-2xl opacity-80"
        style={{ boxShadow: "0 0 60px 10px #fffbe6" }}
      />
      {/* Stars */}
      {stars.map((star, i) => (
        <div
          key={i}
          className="absolute bg-white rounded-full opacity-80"
          style={{
            width: star.width,
            height: star.height,
            top: `${star.top}%`,
            left: `${star.left}%`,
          }}
        />
      ))}
    </div>
  );
};

export default NightSky;
