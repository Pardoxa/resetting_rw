use itertools::Itertools;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{misc::*, Fig4};
use std::{io::Write, process::Command};

pub struct Fig4Python{
    beta_start: f64,
    beta_end: f64,
    samples: usize,
    a: f64
}

impl Fig4Python{
    pub fn exec(&self) -> Vec<(f64, f64)>
    {
        let mut command = Command::new("mfpt_beta.py");
        command
            .args(
                [
                    "greater",
                    "-s",
                    self.beta_start.to_string().as_str(),
                    "-e",
                    self.beta_end.to_string().as_str(),
                    "--samples",
                    self.samples.to_string().as_str(),
                    "-a",
                    self.a.to_string().as_str()
                ]
            );
        let out = command.output().unwrap();
        let output_str = String::from_utf8(out.stdout)
            .unwrap();

        output_str.lines()
            .filter(|line| !line.starts_with('#'))
            .map(
                |line|
                {
                    let mut iter = line.split_ascii_whitespace();
                    let beta: f64 = iter.next().unwrap().parse().unwrap();
                    let mfpt: f64 = iter.next().unwrap().parse().unwrap();
                    (beta, mfpt)
                }
            ).collect_vec()
    }
}


pub fn fig4(options: Fig4)
{
    let range = RatioIter::get_ratio_iter(
        options.a_start, 
        options.a_end, 
        options.steps.get()
    );

    let a_values = range.float_iter().collect_vec();

    let values: Vec<_> = a_values.par_iter()
        .map(
            |&a|
            {
                let cmd = Fig4Python{
                    a,
                    beta_start: options.initial_beta_start,
                    beta_end: options.initial_beta_end,
                    samples: 10000
                };
                
                let beta_mfpt = cmd.exec();
                
                let mut min = f64::INFINITY;
                let mut index_min = 0;
                for (index, (_beta, mfpt)) in beta_mfpt.iter().enumerate()
                {
                    if min > *mfpt{
                        index_min = index;
                        min = *mfpt;
                    }
                }
        
                let beta_min = beta_mfpt[index_min].0;
                let beta_start = beta_min - 0.1;
                let beta_end = beta_min + 0.1;
        
                let cmd = Fig4Python{
                    a,
                    beta_start,
                    beta_end,
                    samples: 200
                };
        
                let beta_mfpt = cmd.exec();
                let file_name = format!("a{a}.dat");
                let header = [
                    "beta",
                    "mfpt"
                ];
                let mut buf = create_buf_with_command_and_version_and_header(
                    file_name, 
                    header
                );
                for (beta, mfpt) in beta_mfpt{
                    writeln!(
                        buf,
                        "{beta} {mfpt}"
                    ).unwrap();
                }

                let gp_name = format!("a{a}.gp");

                let mut gp_writer = create_gnuplot_buf(&gp_name);
                let png = format!("a{a}.png");
                writeln!(gp_writer, "set t pngcairo").unwrap();
                writeln!(gp_writer, "set output '{png}'").unwrap();
                writeln!(gp_writer, "set ylabel 'mfpt'").unwrap();
                writeln!(gp_writer, "set xlabel 'beta'").unwrap();
                writeln!(gp_writer, "set fit quiet").unwrap();
                writeln!(gp_writer, "f(x)=(x-w1)**2*w2+w3").unwrap();
                writeln!(gp_writer, "w1={beta_min}").unwrap();
                writeln!(gp_writer, "w2=0.4").unwrap();
                writeln!(gp_writer, "w3=1.5").unwrap();
                writeln!(gp_writer, "fit f(x) 'a{a}.dat' via w1,w2,w3").unwrap();
                writeln!(gp_writer, "p 'a{a}.dat', f(x)").unwrap();
                writeln!(gp_writer, "print(w1)").unwrap();
                writeln!(gp_writer, "print(w3)").unwrap();
                writeln!(gp_writer, "set output").unwrap();
                drop(gp_writer);
                drop(buf);

                let output = call_gnuplot(&gp_name);

                let s = String::from_utf8(output.stderr)
                        .unwrap();
                
                let mut iter = s.lines();
                let beta: f64 = iter.next().unwrap().parse().unwrap();
                let mfpt: f64 = iter.next().unwrap().parse().unwrap();

                (a, beta, mfpt)

            }
        ).collect();
        create_video(
            "a*png", 
            "test", 
            2, 
            false
        );

        let header = [
            "a",
            "beta",
            "mfpt"
        ];

        let mut buf = create_buf_with_command_and_version_and_header(
            "optimal_analytical_values.dat", 
            header
        );
        for (a, beta, mfpt) in values{
            writeln!(
                buf,
                "{a} {beta} {mfpt}"
            ).unwrap();
        }

}