
import { useMemo, useEffect, useState } from "react";

const STAR_COUNT = 80;
const PARTICLE_COUNT = 25;

const NightSky = () => {
  const [shootingStars, setShootingStars] = useState<Array<{ id: number; delay: number }>>([]);

  // Generate stars only once
  const stars = useMemo(
    () =>
      Array.from({ length: STAR_COUNT }).map(() => ({
        width: Math.random() * 3 + 1,
        height: Math.random() * 3 + 1,
        top: Math.random() * 100,
        left: Math.random() * 100,
        twinkleDelay: Math.random() * 3,
      })),
    []
  );

  // Generate floating particles
  const particles = useMemo(
    () =>
      Array.from({ length: PARTICLE_COUNT }).map(() => ({
        size: Math.random() * 4 + 2,
        top: Math.random() * 100,
        left: Math.random() * 100,
        animationDelay: Math.random() * 10,
        opacity: Math.random() * 0.6 + 0.2,
      })),
    []
  );

  // Generate shooting stars periodically
  useEffect(() => {
    const interval = setInterval(() => {
      const newStar = {
        id: Date.now(),
        delay: Math.random() * 15,
      };
      setShootingStars(prev => [...prev, newStar]);

      // Remove shooting star after animation
      setTimeout(() => {
        setShootingStars(prev => prev.filter(star => star.id !== newStar.id));
      }, 3000);
    }, 8000 + Math.random() * 12000); // Every 8-20 seconds

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="absolute inset-0 -z-10 bg-gradient-to-b from-[#0a1026] via-[#1a223f] to-[#0f1529] overflow-hidden">
      {/* Aurora-like gradient overlay */}
      <div className="absolute inset-0 bg-gradient-to-r from-purple-900/10 via-blue-900/5 to-green-900/10 opacity-60" />

      {/* Moon with glow */}
      <div
        className="absolute top-8 right-16 w-24 h-24 bg-gradient-to-br from-yellow-200 to-yellow-300 rounded-full shadow-2xl opacity-90"
        style={{
          boxShadow: "0 0 80px 15px #fffbe6, 0 0 160px 30px #fffbe6aa",
        }}
      />

      {/* Moon craters */}
      <div className="absolute top-12 right-20 w-3 h-3 bg-yellow-400/40 rounded-full" />
      <div className="absolute top-14 right-24 w-2 h-2 bg-yellow-400/30 rounded-full" />
      <div className="absolute top-16 right-18 w-1.5 h-1.5 bg-yellow-400/50 rounded-full" />

      {/* Stars with twinkling */}
      {stars.map((star, i) => (
        <div
          key={i}
          className="absolute bg-white rounded-full animate-pulse"
          style={{
            width: star.width,
            height: star.height,
            top: `${star.top}%`,
            left: `${star.left}%`,
            animationDelay: `${star.twinkleDelay}s`,
            animationDuration: "2s",
            boxShadow: star.width > 2 ? "0 0 4px rgba(255,255,255,0.8)" : "none",
          }}
        />
      ))}

      {/* Floating particles */}
      {particles.map((particle, i) => (
        <div
          key={`particle-${i}`}
          className="absolute bg-gradient-to-r from-blue-400/60 to-purple-400/60 rounded-full animate-bounce"
          style={{
            width: particle.size,
            height: particle.size,
            top: `${particle.top}%`,
            left: `${particle.left}%`,
            opacity: particle.opacity,
            animationDelay: `${particle.animationDelay}s`,
            animationDuration: "6s",
          }}
        />
      ))}

      {/* Shooting stars */}
      {shootingStars.map(star => (
        <div
          key={star.id}
          className="absolute w-px h-px bg-white animate-ping"
          style={{
            top: `${Math.random() * 30}%`,
            left: "100%",
            animationDelay: `${star.delay}s`,
            animationDuration: "2s",
            transform: "rotate(45deg)",
            boxShadow: "0 0 20px white, 0 0 40px white, 0 0 60px white",
          }}
        >
          <div className="w-8 h-px bg-gradient-to-r from-transparent via-white to-transparent animate-pulse" />
        </div>
      ))}

      {/* Constellation-like patterns */}
      <svg className="absolute inset-0 w-full h-full opacity-20" viewBox="0 0 100 100" preserveAspectRatio="none">
        <defs>
          <linearGradient id="constellation" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#60a5fa" stopOpacity="0.3" />
            <stop offset="50%" stopColor="#a78bfa" stopOpacity="0.5" />
            <stop offset="100%" stopColor="#34d399" stopOpacity="0.3" />
          </linearGradient>
        </defs>

        {/* Ursa Major constellation lines */}
        <path
          d="M15,25 L20,20 L25,18 L30,22 L35,25 L40,20"
          stroke="url(#constellation)"
          strokeWidth="0.2"
          fill="none"
          className="animate-pulse"
          style={{ animationDuration: "8s" }}
        />

        {/* Orion constellation lines */}
        <path
          d="M60,40 L65,35 L70,40 L75,35 L80,40 L85,45"
          stroke="url(#constellation)"
          strokeWidth="0.2"
          fill="none"
          className="animate-pulse"
          style={{ animationDuration: "6s", animationDelay: "2s" }}
        />

        {/* Cassiopeia constellation lines */}
        <path
          d="M10,60 L15,55 L20,60 L25,55 L30,60"
          stroke="url(#constellation)"
          strokeWidth="0.2"
          fill="none"
          className="animate-pulse"
          style={{ animationDuration: "10s", animationDelay: "1s" }}
        />
      </svg>

      {/* Subtle nebula effect */}
      <div className="absolute bottom-0 left-0 w-1/2 h-1/2 bg-gradient-radial from-violet-500/25 via-transparent to-transparent rounded-full blur-3xl animate-pulse" style={{ animationDuration: "12s" }} />
      <div className="absolute top-1/4 right-0 w-1/3 h-1/3 bg-gradient-radial from-sky-400/20 via-transparent to-transparent rounded-full blur-2xl" />
      <div className="absolute inset-x-0 top-20 h-32 bg-gradient-to-b from-cyan-500/15 via-transparent to-transparent blur-3xl" />
      <div className="absolute inset-x-0 top-28 h-24 bg-gradient-to-b from-fuchsia-500/10 via-transparent to-transparent blur-3xl" />
    </div>
  );
};

export default NightSky;
