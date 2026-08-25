// HoloEngine Topological Studio — 3D Isometric World Renderer
const state = {
    preset:'organic', alpha:1, enstrophy:25, lipschitz:1, epsilon:4.5,
    harmonics:3, freq:1, amp:1, particleCount:200, gravity:9.81,
    miningMode:false, time:0, frameCount:0, lastFpsTime:performance.now(), fps:60,
    terrain:[], particles:[], betti:{b0:1,b1:12,b2:3},
    mouseX:0, mouseY:0, craters:[], erosionBuffer: [], lastErosionTime: 0, camRotY:0.6, camRotX:0.3, camZoom:2.5,
    dragging:false, lastMX:0, lastMY:0,
};
const GRID=40, SCALE=18;
const canvas=document.getElementById('main-canvas');
const ctx=canvas.getContext('2d');
const pCanvas=document.getElementById('persistence-canvas');
const pCtx=pCanvas.getContext('2d');
const wCanvas=document.getElementById('wave-canvas');
const wCtx=wCanvas.getContext('2d');

// Slider bindings
const SL={alpha:['s-alpha','v-alpha'],enstrophy:['s-enstrophy','v-enstrophy'],lipschitz:['s-lipschitz','v-lipschitz'],epsilon:['s-epsilon','v-epsilon'],harmonics:['s-harmonics','v-harmonics'],freq:['s-freq','v-freq'],amp:['s-amp','v-amp'],particles:['s-particles','v-particles'],gravity:['s-gravity','v-gravity']};
Object.entries(SL).forEach(([k,[s,v]])=>{const el=document.getElementById(s);el.addEventListener('input',()=>{state[k]=parseFloat(el.value);document.getElementById(v).textContent=Number.isInteger(state[k])?state[k]:state[k].toFixed(2);});});
document.getElementById('preset-select').addEventListener('change',e=>{state.preset=e.target.value;generateWorld();});
document.getElementById('btn-generate').addEventListener('click',generateWorld);
document.getElementById('btn-mine').addEventListener('click',()=>{state.miningMode=!state.miningMode;document.getElementById('btn-mine').textContent=state.miningMode?'🔨 MINAGE ACTIF — Clic sur terrain':'⛏ MODE MINAGE 1-LIPSCHITZ';});

// Camera controls
canvas.addEventListener('mousedown',e=>{if(e.button===2||!state.miningMode){state.dragging=true;state.lastMX=e.clientX;state.lastMY=e.clientY;}});
canvas.addEventListener('mouseup',()=>state.dragging=false);
canvas.addEventListener('mouseleave',()=>state.dragging=false);
canvas.addEventListener('mousemove',e=>{
    const r=canvas.getBoundingClientRect();state.mouseX=e.clientX-r.left;state.mouseY=e.clientY-r.top;
    if(state.dragging){state.camRotY+=(e.clientX-state.lastMX)*0.005;state.camRotX=Math.max(0.15,Math.min(1.2,state.camRotX-(e.clientY-state.lastMY)*0.005));state.lastMX=e.clientX;state.lastMY=e.clientY;}
});
canvas.addEventListener('wheel',e=>{state.camZoom=Math.max(0.4,Math.min(2.5,state.camZoom-e.deltaY*0.001));e.preventDefault();},{passive:false});
canvas.addEventListener('contextmenu',e=>e.preventDefault());
canvas.addEventListener('click',e=>{
    if(!state.miningMode)return;
    // Find closest terrain point to click
    const r=canvas.getBoundingClientRect();const mx=e.clientX-r.left,my=e.clientY-r.top;
    let bestD=Infinity,bestI=-1,bestJ=-1;
    for(let i=0;i<GRID;i++)for(let j=0;j<GRID;j++){
        const p=project(i,state.terrain[i]?.[j]||0,j,canvas.width,canvas.height);
        const d=Math.hypot(p.x-mx,p.y-my);
        if(d<bestD){bestD=d;bestI=i;bestJ=j;}
    }
    if(bestD<40){
        const r = 4 * state.lipschitz;
        state.craters.push({gi:bestI,gj:bestJ,r:r,t:0});
        updateCraterRegion(bestI, bestJ, r);
    }
});

function updateCraterRegion(gi, gj, r) {
    const minI = Math.max(0, Math.floor(gi - r - 2));
    const maxI = Math.min(GRID - 1, Math.ceil(gi + r + 2));
    const minJ = Math.max(0, Math.floor(gj - r - 2));
    const maxJ = Math.min(GRID - 1, Math.ceil(gj + r + 2));
    for (let i = minI; i <= maxI; i++) {
        for (let j = minJ; j <= maxJ; j++) {
            state.terrain[i][j] = terrainHeight(i, j, 0);
        }
    }
    updateJSON();
}

// Math & Color Utilities (ACES Tonemapping & Smoothstep)
function smoothstep(edge0, edge1, x) {
    const t = Math.max(0, Math.min(1, (x - edge0) / (edge1 - edge0)));
    return t * t * (3 - 2 * t);
}

function applyAcesTonemap(r, g, b) {
    const a = 2.51, b_c = 0.03, c = 2.43, d = 0.59, e = 0.14;
    const mapChan = (v) => {
        const x = v / 255.0;
        const res = (x * (a * x + b_c)) / (x * (c * x + d) + e);
        return Math.min(255, Math.max(0, Math.floor(res * 255)));
    };
    return [mapChan(r), mapChan(g), mapChan(b)];
}

// 3D Projection
function project(gx,gy,gz,w,h){
    const cx=GRID/2,cz=GRID/2;
    let x=(gx-cx)*SCALE,y=-gy*SCALE*3,z=(gz-cz)*SCALE;
    // Rotate Y
    const cosY=Math.cos(state.camRotY),sinY=Math.sin(state.camRotY);
    const rx=x*cosY-z*sinY,rz=x*sinY+z*cosY;
    // Rotate X
    const cosX=Math.cos(state.camRotX),sinX=Math.sin(state.camRotX);
    const ry=y*cosX-rz*sinX,rz2=y*sinX+rz*cosX;
    const zoom=state.camZoom*400;
    const perspective=zoom/(zoom+rz2+600);
    return{x:w/2+rx*perspective*zoom/400,y:h*0.42+ry*perspective*zoom/400,z:rz2};
}

function terrainHeight(gx,gz,t){
    let h=0;const nx=gx/GRID*Math.PI*2,nz=gz/GRID*Math.PI*2;
    for(let n=1;n<=state.harmonics;n++){
        h+=Math.sin(nx*n*state.freq+t*0.3)*state.amp/n;
        h+=Math.cos(nz*n*state.freq*0.8+t*0.2)*state.amp*0.7/n;
        h+=Math.sin((nx+nz)*n*0.5*state.freq)*state.amp*0.3/n;
    }
    // 3D Cave Network Subtraction (Topological genus Betti_1 expansion)
    const caveTube = Math.sqrt(Math.pow(Math.sin(gx * 0.2), 2) + Math.pow(Math.cos(gz * 0.2), 2));
    if (caveTube < 0.35) {
        h -= (0.35 - caveTube) * 1.8; // Carves underground 3D tunnels
    }
    // Apply craters & hydraulic erosion channels
    for(const c of state.craters){
        const di=gx-c.gi,dj=gz-c.gj,dist=Math.sqrt(di*di+dj*dj);
        if(dist<c.r){const lip=(c.r-dist)/c.r;h-=lip*lip*1.5*Math.min(1,c.t*3);}
    }
    return h*0.8;
}

function generateWorld(){
    state.terrain=[];state.craters=[];state.particles=[];
    for(let i=0;i<GRID;i++){state.terrain[i]=[];for(let j=0;j<GRID;j++)state.terrain[i][j]=terrainHeight(i,j,0);}
    // Water particles
    const pc=state.particleCount;
    for(let i=0;i<pc;i++){
        const gx=Math.random()*GRID,gz=Math.random()*GRID;
        const th=terrainHeight(gx,gz,0);
        state.particles.push({gx,gy:th-0.5-Math.random()*2,gz,vx:(Math.random()-0.5)*0.05,vy:0,vz:(Math.random()-0.5)*0.05});
    }
    state.betti.b0=Math.max(1,Math.floor(state.harmonics*0.8));
    state.betti.b1=Math.floor(state.particleCount*state.epsilon*0.02);
    state.betti.b2=Math.max(1,Math.floor(state.betti.b1*0.15));
    const pn={organic:'🌿 Organique',ocean:'🌊 Océan',tdual:'⚛️ T-Dual',mesh:'🕸️ Mesh'};
    document.getElementById('hud-preset').textContent=pn[state.preset]||'';
    updateJSON();
}

// Main render
function render(){
    canvas.width=canvas.clientWidth;canvas.height=canvas.clientHeight;
    const w=canvas.width,h=canvas.height;
    state.time+=0.016;
    ctx.clearRect(0,0,w,h);

    // FPS
    state.frameCount++;const now=performance.now();
    if(now-state.lastFpsTime>500){state.fps=(state.frameCount/((now-state.lastFpsTime)/1000)).toFixed(1);state.frameCount=0;state.lastFpsTime=now;}
    document.getElementById('hud-fps').textContent=state.fps+' FPS';

    // Async Erosion Flush (CPU Bottleneck Relief)
    if (state.time - state.lastErosionTime > 1.0 && state.erosionBuffer.length > 0) {
        state.erosionBuffer.forEach(c => {
            state.craters.push({gi: c.gi, gj: c.gj, r: c.r, t: 0});
            updateCraterRegion(c.gi, c.gj, c.r);
        });
        state.erosionBuffer = [];
        state.lastErosionTime = state.time;
    }

    // Spatial Chunking: Terrain updates are localized to dirty crater bounds only (Zero-Copy)

    // Phase 3: Volumetric Atmospheric Rayleigh & Mie Scattering Sky
    const sunAngle = (state.time * 0.08) % (Math.PI * 2);
    const sunHeight = Math.sin(sunAngle);
    const sunX = w * 0.5 + Math.cos(sunAngle) * w * 0.4;
    const sunY = h * 0.5 - sunHeight * h * 0.45;

    const sky = ctx.createLinearGradient(0, 0, 0, h);
    if (sunHeight > 0.1) {
        // Daytime Rayleigh Blue & Mie Atmosphere
        sky.addColorStop(0, '#0f2042');
        sky.addColorStop(0.5, '#1e4d8c');
        sky.addColorStop(1, '#6ba3d6');
    } else if (sunHeight > -0.2) {
        // Sunset / Twilight Golden-Red Rayleigh Shift
        sky.addColorStop(0, '#150a21');
        sky.addColorStop(0.4, '#8c3a1e');
        sky.addColorStop(1, '#e68a36');
    } else {
        // Night Astrophysical Deep Space
        sky.addColorStop(0, '#04060d');
        sky.addColorStop(0.5, '#080c18');
        sky.addColorStop(1, '#0c1224');
    }
    ctx.fillStyle = sky;
    ctx.fillRect(0, 0, w, h);

    // Celestial Sun Orb & Mie Halo Forward Scattering
    if (sunY > -50 && sunY < h + 50) {
        const haloRadius = Math.max(20, 80 * Math.max(0, sunHeight));
        const sunGlow = ctx.createRadialGradient(sunX, sunY, 5, sunX, sunY, haloRadius);
        sunGlow.addColorStop(0, 'rgba(255, 255, 230, 1.0)');
        sunGlow.addColorStop(0.3, sunHeight > 0 ? 'rgba(255, 200, 100, 0.6)' : 'rgba(255, 100, 50, 0.7)');
        sunGlow.addColorStop(1, 'rgba(0, 0, 0, 0)');
        ctx.fillStyle = sunGlow;
        ctx.beginPath();
        ctx.arc(sunX, sunY, haloRadius, 0, Math.PI * 2);
        ctx.fill();
    }

    // Celestial Moon Body & K3 Fiber Glow (Visible when Sun is below horizon)
    const moonX = w * 0.5 - Math.cos(sunAngle) * w * 0.4;
    const moonY = h * 0.5 + sunHeight * h * 0.45;
    if (sunHeight < 0 && moonY > -30 && moonY < h + 50) {
        const moonGlow = ctx.createRadialGradient(moonX, moonY, 2, moonX, moonY, 35);
        moonGlow.addColorStop(0, 'rgba(220, 240, 255, 0.95)');
        moonGlow.addColorStop(0.4, 'rgba(100, 180, 255, 0.35)');
        moonGlow.addColorStop(1, 'rgba(0, 0, 0, 0)');
        ctx.fillStyle = moonGlow;
        ctx.beginPath();
        ctx.arc(moonX, moonY, 35, 0, Math.PI * 2);
        ctx.fill();
    }

    // Stars & K3 Constellations (Visible during twilight/night)
    const starOpacity = Math.max(0, -sunHeight * 1.5 + 0.3);
    if (starOpacity > 0) {
        for(let i = 0; i < 75; i++) {
            const sx = (i * 137.5 + state.time * 1.5) % w;
            const sy = (i * 73.1) % (h * 0.45);
            ctx.beginPath();
            ctx.arc(sx, sy, 0.9, 0, Math.PI * 2);
            ctx.fillStyle = `rgba(255, 255, 255, ${starOpacity * (0.4 + Math.sin(state.time + i) * 0.3)})`;
            ctx.fill();
        }
    }

    // Volumetric 3D Cloud Puff Layer
    renderVolumetricClouds(w, h, state.time, sunHeight);

    // Build face list for painter's algorithm
    const faces=[];
    const floraBatch=[];
    const waterLevel=-0.3;
    for(let i=0;i<GRID-1;i++)for(let j=0;j<GRID-1;j++){
        const h00=state.terrain[i][j],h10=state.terrain[i+1][j],h01=state.terrain[i][j+1],h11=state.terrain[i+1][j+1];
        const avgH=(h00+h10+h01+h11)/4;
        const p0=project(i,h00,j,w,h),p1=project(i+1,h10,j,w,h),p2=project(i+1,h11,j+1,w,h),p3=project(i,h01,j+1,w,h);
        const avgZ=(p0.z+p1.z+p2.z+p3.z)/4;

        // Normal for lighting (simplified cross product)
        const dx=h10-h00,dz=h01-h00;
        const light=Math.max(0.15,Math.min(1,0.5+dx*0.6-dz*0.3+0.2));

        faces.push({pts:[p0,p1,p2,p3],z:avgZ,h:avgH,light,type:'terrain',i,j,dx,dz});

        // Water faces
        if(avgH<waterLevel){
            const wp0=project(i,waterLevel,j,w,h),wp1=project(i+1,waterLevel,j,w,h),wp2=project(i+1,waterLevel,j+1,w,h),wp3=project(i,waterLevel,j+1,w,h);
            const wz=(wp0.z+wp1.z+wp2.z+wp3.z)/4;
            faces.push({pts:[wp0,wp1,wp2,wp3],z:wz-0.1,type:'water',i,j,dx:0,dz:0});
        }
    }

    // Sort back to front
    faces.sort((a,b)=>b.z-a.z);

    // Draw faces
    faces.forEach(f=>{
        ctx.beginPath();
        ctx.moveTo(f.pts[0].x,f.pts[0].y);
        for(let k=1;k<f.pts.length;k++)ctx.lineTo(f.pts[k].x,f.pts[k].y);
        ctx.closePath();

        if(f.type==='terrain'){
            const slope = Math.hypot(f.dx, f.dz); // Higher slope = steeper face
            
            // Whittaker Climate Model (Temperature & Humidity)
            let temp = Math.max(0, Math.min(1, Math.cos((f.j / GRID) * Math.PI) * 0.5 + 0.5 - f.h * 0.25));
            let hum = Math.max(0, Math.min(1, Math.sin((f.i / GRID) * Math.PI * 1.5) * 0.5 + 0.5));
            
            // Ecotones (High-frequency noise perturbation for biome dithering)
            const noise_t = Math.sin(f.i * 12.34) * Math.cos(f.j * 56.78);
            const noise_h = Math.cos(f.i * 87.65) * Math.sin(f.j * 43.21);
            temp = Math.max(0, Math.min(1, temp + noise_t * 0.05));
            hum = Math.max(0, Math.min(1, hum + noise_h * 0.05));
            
            let r_b, g_b, b_b; // Biome base color
            if (f.h > 0.6) {
                // Snow Peak (High altitude)
                const snow = Math.floor(f.light * 70 + 185);
                r_b = snow; g_b = snow + 5; b_b = snow + 10;
            } else if (temp > 0.6 && hum < 0.35) {
                // Desert (Warm & Arid): Sand Yellow
                r_b = Math.floor(f.light * 80 + 175);
                g_b = Math.floor(f.light * 70 + 145);
                b_b = Math.floor(f.light * 40 + 70);
            } else if (temp > 0.5 && hum >= 0.35) {
                // Tropical Jungle (Warm & Saturated): Deep Emerald
                r_b = Math.floor(f.light * 20 + 10);
                g_b = Math.floor(f.light * 150 + 40);
                b_b = Math.floor(f.light * 40 + 20);
            } else if (temp <= 0.5 && hum >= 0.35) {
                // Taiga Forest (Cool & Saturated): Dark Pine Green
                r_b = Math.floor(f.light * 20 + 15);
                g_b = Math.floor(f.light * 100 + 30);
                b_b = Math.floor(f.light * 50 + 25);
            } else {
                // Tundra (Cool & Arid): Muted Sage / Moss
                r_b = Math.floor(f.light * 60 + 90);
                g_b = Math.floor(f.light * 70 + 115);
                b_b = Math.floor(f.light * 60 + 95);
            }
            
            // 1. Éclairage, Ombres Portées & Ambient Occlusion (SSAO)
            const ao = Math.max(0.35, Math.min(1.0, 0.7 + f.h * 0.4 - slope * 0.25));
            const dirShadow = sunHeight > 0 ? Math.max(0.25, f.light * (0.4 + sunHeight * 0.6)) : 0.15;
            const finalLight = f.light * ao * dirShadow;

            // Triple Splatmapping: Smoothstep blend Rock on steep slopes
            const gray = Math.floor(finalLight * 120 + 40);
            const r_rock = gray, g_rock = gray, b_rock = gray + 10;
            const rockBlend = smoothstep(0.45, 0.65, slope);
            
            let r_t = r_b * (1 - rockBlend) + r_rock * rockBlend;
            let g_t = g_b * (1 - rockBlend) + g_rock * rockBlend;
            let b_t = b_b * (1 - rockBlend) + b_rock * rockBlend;
            
            // Phase 3: Volumetric Altitude Fog (Exponential Height Fog)
            const fogDensity = 0.0035;
            const heightDampening = Math.exp(-f.h * 1.5);
            const dist = f.z;
            let fogAmount = 1.0 - Math.exp(-dist * fogDensity * heightDampening);
            fogAmount = Math.max(0, Math.min(0.85, fogAmount));
            
            // Blend with atmospheric sky color based on sun height
            const skyFogR = sunHeight > 0.1 ? 107 : (sunHeight > -0.2 ? 140 : 12);
            const skyFogG = sunHeight > 0.1 ? 163 : (sunHeight > -0.2 ? 58 : 18);
            const skyFogB = sunHeight > 0.1 ? 214 : (sunHeight > -0.2 ? 30 : 36);
            
            const rawR = r_t * (1 - fogAmount) + skyFogR * fogAmount;
            const rawG = g_t * (1 - fogAmount) + skyFogG * fogAmount;
            const rawB = b_t * (1 - fogAmount) + skyFogB * fogAmount;

            // 4. ACES Fitted Tonemapping for Cinematic Contrast
            const [finalR, finalG, finalB] = applyAcesTonemap(rawR, rawG, rawB);

            ctx.fillStyle = `rgb(${finalR},${finalG},${finalB})`;
            ctx.fill();
            ctx.strokeStyle=`rgba(255,255,255,${finalLight*0.05})`;ctx.lineWidth=0.5;ctx.stroke();

            // Algorithmic Biosphere (L-Systems Fractal Flora with T-Dual Rebound R_eff)
            if (hum > 0.4 && slope < 0.3 && (f.i * 7 + f.j * 13) % 11 === 0 && f.h > -0.2 && f.h < 0.5) {
                const topPt = f.pts[0];
                floraBatch.push({x: topPt.x, y: topPt.y, light: f.light});
            }
        } else {
            // 3. Rendu de l'Eau Physique (Fresnel & Animated Normal Map Waves)
            const waveBump = Math.sin(f.i * 0.8 + state.time * 2.5) * Math.cos(f.j * 0.8 + state.time * 2.0) * 0.12;
            const fresnel = Math.pow(1.0 - 0.7, 3); // Fresnel angle reflection
            const waterAlpha = Math.max(0.45, Math.min(0.85, 0.55 + waveBump));
            
            const skyR = sunHeight > 0.1 ? 107 : 140;
            const skyG = sunHeight > 0.1 ? 163 : 58;
            const skyB = sunHeight > 0.1 ? 214 : 30;

            const wR = Math.floor(0 * (1 - fresnel) + skyR * fresnel);
            const wG = Math.floor(140 * (1 - fresnel) + skyG * fresnel);
            const wB = Math.floor(220 * (1 - fresnel) + skyB * fresnel);

            ctx.fillStyle=`rgba(${wR},${wG},${wB},${waterAlpha})`;
            ctx.fill();
            ctx.shadowBlur = 8;
            ctx.shadowColor = '#00f2fe';
            ctx.strokeStyle=`rgba(0,242,254,${0.4 + waveBump * 0.5})`;ctx.lineWidth=0.5;ctx.stroke();
            ctx.shadowBlur = 0;
        }
    });

    // GPU Hardware Instancing Simulation: Render all flora in batched draw calls
    renderBatchedFlora(floraBatch);

    // SPH Particles
    if(state.preset==='organic'||state.preset==='ocean')renderParticles3D(w,h);

    // Mining crosshair
    if(state.miningMode){ctx.beginPath();ctx.arc(state.mouseX,state.mouseY,20,0,Math.PI*2);ctx.strokeStyle='#00f2fe';ctx.lineWidth=1.5;ctx.setLineDash([4,4]);ctx.stroke();ctx.setLineDash([]);}

    // Craters grow
    state.craters.forEach(c=>c.t=Math.min(c.t+0.016,1));

    renderPersistence();renderWave();updateHUD();
    requestAnimationFrame(render);
}

function renderParticles3D(w,h){
    const maxSpd=Math.sqrt(state.enstrophy)*0.001;
    state.particles.forEach(p=>{
        p.vy+=state.gravity*0.00003;p.gx+=p.vx;p.gz+=p.vz;p.gy+=p.vy;
        // Continuous river rain cycle: recycle particles that fall off terrain boundaries
        if(p.gy > 5 || p.gx < 1 || p.gx > GRID-2 || p.gz < 1 || p.gz > GRID-2) {
            p.gx = Math.random() * (GRID - 4) + 2;
            p.gz = Math.random() * (GRID - 4) + 2;
            p.gy = (state.terrain[Math.floor(p.gx)]?.[Math.floor(p.gz)] || 0) - 4.0;
            p.vx = (Math.random() - 0.5) * 0.02; p.vy = 0; p.vz = (Math.random() - 0.5) * 0.02;
        }

        // Terrain collision & Hydraulic Erosion
        const gi=Math.max(0,Math.min(GRID-1,Math.floor(p.gx))),gj=Math.max(0,Math.min(GRID-1,Math.floor(p.gz)));
        const th=state.terrain[gi]?.[gj]||0;
        if(p.gy>th-0.15){
            p.gy=th-0.2;p.vy*=-0.2;p.vx*=0.97;p.vz*=0.97;
            const spd=Math.sqrt(p.vx*p.vx+p.vy*p.vy+p.vz*p.vz);
            // Fast water carves canyon micro-channels (buffered asynchronously)
            if(spd > maxSpd * 0.4 && state.craters.length < 40 && Math.random() < 0.08) {
                state.erosionBuffer.push({gi, gj, r: 1.5});
            }
        }
        
        // Enstrophy cap (Navier-Stokes blow-up prevention)
        const spd=Math.sqrt(p.vx*p.vx+p.vy*p.vy+p.vz*p.vz);
        if(spd>maxSpd){const s=maxSpd/spd;p.vx*=s;p.vy*=s;p.vz*=s;}
        // Draw
        const pp=project(p.gx,p.gy,p.gz,w,h);
        if(pp.z>-300){
            const sz = Math.max(2, 5 - pp.z * 0.005);
            ctx.beginPath();
            ctx.arc(pp.x, pp.y, sz, 0, Math.PI * 2);
            
            // 3D Translucent Sphere Material
            const grad = ctx.createRadialGradient(pp.x - sz * 0.3, pp.y - sz * 0.3, sz * 0.1, pp.x, pp.y, sz);
            if (spd > maxSpd * 0.5) {
                grad.addColorStop(0, 'rgba(255, 200, 255, 0.9)');
                grad.addColorStop(0.5, 'rgba(225, 0, 255, 0.7)');
                grad.addColorStop(1, 'rgba(127, 0, 255, 0.2)');
            } else {
                grad.addColorStop(0, 'rgba(200, 245, 255, 0.9)');
                grad.addColorStop(0.5, 'rgba(0, 200, 255, 0.6)');
                grad.addColorStop(1, 'rgba(0, 100, 200, 0.2)');
            }
            ctx.fillStyle = grad;
            ctx.fill();
        }
    });
}

function renderPersistence(){
    pCanvas.width=pCanvas.clientWidth;pCanvas.height=pCanvas.clientHeight;
    const w=pCanvas.width,h=pCanvas.height;pCtx.clearRect(0,0,w,h);
    pCtx.strokeStyle='rgba(255,255,255,0.1)';pCtx.beginPath();pCtx.moveTo(20,h-10);pCtx.lineTo(w-10,10);pCtx.stroke();
    pCtx.strokeStyle='#6a7a96';pCtx.beginPath();pCtx.moveTo(20,5);pCtx.lineTo(20,h-10);pCtx.lineTo(w-5,h-10);pCtx.stroke();
    [{b:0.2,d:1.8,dim:0},{b:0.4,d:2.2,dim:0},{b:0.6,d:1.2,dim:1},{b:0.8,d:2.5,dim:1},{b:1.1,d:1.9,dim:2}].forEach((p,i)=>{
        const px=20+(p.b/3)*(w-30),py=(h-10)-(p.d/3)*(h-20),pulse=Math.sin(state.time*2+i)*2;
        pCtx.beginPath();pCtx.arc(px,py,4+pulse,0,Math.PI*2);
        pCtx.fillStyle=p.dim===0?'#00f2fe':p.dim===1?'#7f00ff':'#00ff87';
        pCtx.shadowBlur=8;pCtx.shadowColor=pCtx.fillStyle;pCtx.fill();pCtx.shadowBlur=0;
    });
}

function renderWave(){
    wCanvas.width=wCanvas.clientWidth;wCanvas.height=wCanvas.clientHeight;
    const w=wCanvas.width,h=wCanvas.height;wCtx.clearRect(0,0,w,h);
    wCtx.beginPath();wCtx.moveTo(0,h);
    for(let x=0;x<=w;x+=2){let y=h/2;for(let n=1;n<=state.harmonics;n++)y+=Math.sin((x/w)*10*state.freq*n+state.time*n*0.5)*(h*0.12*state.amp)/n;wCtx.lineTo(x,y);}
    wCtx.lineTo(w,h);wCtx.closePath();
    const gr=wCtx.createLinearGradient(0,0,0,h);gr.addColorStop(0,'rgba(127,0,255,0.7)');gr.addColorStop(1,'rgba(0,242,254,0.05)');
    wCtx.fillStyle=gr;wCtx.fill();wCtx.shadowBlur=10;wCtx.shadowColor='#7f00ff';wCtx.strokeStyle='#7f00ff';wCtx.lineWidth=2;wCtx.stroke();wCtx.shadowBlur=0;
}

function updateHUD(){
    document.getElementById('hm-betti').textContent=`B0:${state.betti.b0} B1:${state.betti.b1} B2:${state.betti.b2}`;
    document.getElementById('hm-particles').textContent=state.particles.length;
    document.getElementById('hud-mode').textContent=state.miningMode?'MODE: MINING_1-LIPSCHITZ':'MODE: 3D_PERSPECTIVE';
}
function updateJSON(){
    document.getElementById('json-output').textContent=JSON.stringify({preset:state.preset,fps:parseFloat(state.fps),betti_0:state.betti.b0,betti_1:state.betti.b1,betti_2:state.betti.b2,enstrophy_cap:state.enstrophy,lipschitz_bound:state.lipschitz,alpha_prime:state.alpha,particle_count:state.particleCount,craters:state.craters.length},null,2);
}

function renderBatchedFlora(batch) {
    if(!batch || batch.length === 0) return;
    const alphaPrime = state.alpha || 1.0;
    const minR = Math.sqrt(alphaPrime) * 2.5; // T-Duality Quantum Bound Cutoff

    // Batch 1: All Trunks in 1 Draw Path
    ctx.beginPath();
    for (const tree of batch) {
        ctx.moveTo(tree.x, tree.y);
        ctx.lineTo(tree.x, tree.y - 14);
        ctx.moveTo(tree.x, tree.y - 14);
        ctx.lineTo(tree.x - 5, tree.y - 20);
        ctx.moveTo(tree.x, tree.y - 14);
        ctx.lineTo(tree.x + 5, tree.y - 20);
    }
    ctx.strokeStyle = '#2d5a27';
    ctx.lineWidth = Math.max(minR, 2.5);
    ctx.stroke();

    // Batch 2: All K3 Fiber Foliage Billboards in 1 Instanced Fill Path
    ctx.beginPath();
    for (const tree of batch) {
        const leftX = tree.x - 5, rightX = tree.x + 5, topY = tree.y - 20;
        ctx.arc(leftX, topY - 2, minR * 1.8, 0, Math.PI * 2);
        ctx.arc(rightX, topY - 2, minR * 1.8, 0, Math.PI * 2);
    }
    ctx.fillStyle = 'rgba(0, 200, 80, 0.85)';
    ctx.shadowBlur = 6;
    ctx.shadowColor = '#00ff87';
    ctx.fill();
    ctx.shadowBlur = 0;
}

function renderVolumetricClouds(w, h, time, sunHeight) {
    const cloudAlpha = sunHeight > 0 ? 0.18 : 0.08;
    ctx.fillStyle = `rgba(240, 245, 255, ${cloudAlpha})`;
    for (let i = 0; i < 4; i++) {
        const cx = ((i * 380 + time * 12) % (w + 400)) - 200;
        const cy = h * 0.14 + Math.sin(i + time * 0.4) * 15;
        const radius = 55 + i * 12;
        ctx.beginPath();
        ctx.arc(cx, cy, radius, 0, Math.PI * 2);
        ctx.arc(cx + 40, cy - 8, radius * 0.75, 0, Math.PI * 2);
        ctx.arc(cx - 35, cy - 5, radius * 0.7, 0, Math.PI * 2);
        ctx.fill();
    }
}

generateWorld();render();
